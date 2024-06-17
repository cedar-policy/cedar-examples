/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use thiserror::Error;

use cedar_policy::{Entities, EvaluationError, Schema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    context::Error,
    objects::{Application, List, Team, User, UserOrTeam},
    util::{make_list_euid, make_team_euid, make_user_euid, TeamUid, UserOrTeamUid, UserUid},
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EntityStore {
    users: HashMap<String, User>,
    teams: HashMap<String, Team>,
    lists: HashMap<String, List>,
    app: Application,
}

impl EntityStore {
    pub fn get_lists(&self) -> impl Iterator<Item = &List> {
        self.lists.values()
    }

    pub fn as_entities(&self, schema: &Schema) -> Entities {
        let users = self.users.values().map(|user| user.clone().into());
        let teams = self.teams.values().map(|team| team.clone().into());
        let lists = self.lists.values().map(|list| list.clone().into());
        let app = std::iter::once(self.app.clone().into());
        let all = users.chain(teams).chain(lists).chain(app);
        Entities::from_entities(all, Some(schema)).unwrap()
    }

    pub fn fresh_eid(&self) -> String {
        loop {
            let new_eid: String = Uuid::new_v4().to_string();
            if !self.euid_exists(&new_eid) {
                return new_eid;
            }
        }
    }

    fn euid_exists(&self, eid: &str) -> bool {
        self.lists.contains_key(eid)
            || self.teams.contains_key(eid)
            || self.users.contains_key(eid)
            || self.app.euid().id().as_ref() == eid
    }

    pub fn insert_user(&mut self, e: User) {
        self.users.insert(e.eid(), e);
    }

    pub fn insert_team(&mut self, e: Team) {
        self.teams.insert(e.eid(), e);
    }

    pub fn insert_list(&mut self, e: List) {
        self.lists.insert(e.eid(), e);
    }

    pub fn list(&mut self, e: &str) -> Result<(), Error> {
        if self.lists.contains_key(e) {
            self.lists.remove(e);
            Ok(())
        } else {
            Err(Error::NoSuchEntity(make_list_euid(e)))
        }
    }

    pub fn get_user(&self, eid: &str) -> Result<&User, Error> {
        self.users
            .get(eid)
            .ok_or_else(|| Error::no_such_entity(make_user_euid(eid)))
    }

    pub fn get_user_mut(&mut self, eid: &str) -> Result<&mut User, Error> {
        self.users
            .get_mut(eid)
            .ok_or_else(|| Error::no_such_entity(make_user_euid(eid)))
    }

    pub fn get_team(&self, eid: &str) -> Result<&Team, Error> {
        self.teams
            .get(eid)
            .ok_or_else(|| Error::no_such_entity(make_team_euid(eid)))
    }

    pub fn get_team_mut(&mut self, eid: &str) -> Result<&mut Team, Error> {
        self.teams
            .get_mut(eid)
            .ok_or_else(|| Error::no_such_entity(make_team_euid(eid)))
    }

    pub fn get_user_or_team_mut(
        &mut self,
        euid: &UserOrTeamUid,
    ) -> Result<&mut dyn UserOrTeam, Error> {
        let euid_ref = euid.as_ref();
        if let Ok(t) = euid.clone().try_into() {
            let t: TeamUid = t;
            if let Some(t) = self.teams.get_mut(t.as_ref().id().as_ref()) {
                return Ok(t);
            }
        } else if let Ok(u) = euid.clone().try_into() {
            let u: UserUid = u;
            if let Some(u) = self.users.get_mut(u.as_ref().id().as_ref()) {
                return Ok(u);
            }
        }
        Err(Error::no_such_entity(euid_ref.clone()))
    }

    pub fn delete_list(&mut self, eid: &str) -> Result<(), Error> {
        if self.lists.contains_key(eid) {
            self.lists.remove(eid);
            Ok(())
        } else {
            Err(Error::no_such_entity(make_list_euid(eid)))
        }
    }

    pub fn get_list(&self, eid: &str) -> Result<&List, Error> {
        self.lists
            .get(eid)
            .ok_or_else(|| Error::no_such_entity(make_list_euid(eid)))
    }

    pub fn get_list_mut(&mut self, eid: &str) -> Result<&mut List, Error> {
        self.lists
            .get_mut(eid)
            .ok_or_else(|| Error::no_such_entity(make_list_euid(eid)))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    List,
    User,
    Team,
    Application,
}

#[derive(Debug, Clone, Error)]
pub enum EntityDecodeError {
    #[error("The following required attribute was missing: {0}")]
    MissingAttr(&'static str),
    #[error("Evaluation Failed: {0}")]
    Eval(#[from] EvaluationError),
    #[error("Field {0} was wrong typed. Expected {0}")]
    WrongType(&'static str, &'static str),
    #[error("Enum was not one of required fields. Enum{enumeration}, Got {got}")]
    BadEnum {
        enumeration: &'static str,
        got: String,
    },
}
