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

use cedar_policy::{Entities, EntityId, EntityTypeName, EvaluationError, Schema};
use serde::{Deserialize, Serialize};

use crate::{
    context::Error,
    objects::{Application, List, Team, User, UserOrTeam},
    util::{EntityUid, ListUid, TeamUid, UserOrTeamUid, UserUid},
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EntityStore {
    users: HashMap<EntityUid, User>,
    teams: HashMap<EntityUid, Team>,
    lists: HashMap<EntityUid, List>,
    app: Application,
    #[serde(skip)]
    uid: usize,
}

impl EntityStore {
    pub fn euids(&self) -> impl Iterator<Item = &EntityUid> {
        self.users
            .keys()
            .chain(self.teams.keys())
            .chain(self.lists.keys())
            .chain(std::iter::once(self.app.euid()))
    }

    pub fn as_entities(&self, schema: &Schema) -> Entities {
        let users = self.users.values().map(|user| user.clone().into());
        let teams = self.teams.values().map(|team| team.clone().into());
        let lists = self.lists.values().map(|list| list.clone().into());
        let app = std::iter::once(self.app.clone().into());
        let all = users.chain(teams).chain(lists).chain(app);
        Entities::from_entities(all, Some(schema)).unwrap()
    }

    pub fn fresh_euid<T: TryFrom<EntityUid>>(&mut self, ty: EntityTypeName) -> Result<T, T::Error> {
        loop {
            let new_uid: EntityId = format!("{}", self.uid).parse().unwrap();
            self.uid += 1;
            let euid = cedar_policy::EntityUid::from_type_name_and_id(ty.clone(), new_uid).into();
            if !self.euid_exists(&euid) {
                return T::try_from(euid);
            }
        }
    }

    fn euid_exists(&self, euid: &EntityUid) -> bool {
        self.lists.contains_key(euid)
            || self.teams.contains_key(euid)
            || self.users.contains_key(euid)
            || self.app.euid() == euid
    }

    pub fn insert_user(&mut self, e: User) {
        self.users.insert(e.uid().clone().into(), e);
    }

    pub fn insert_team(&mut self, e: Team) {
        self.teams.insert(e.uid().clone().into(), e);
    }

    pub fn insert_list(&mut self, e: List) {
        self.lists.insert(e.uid().clone().into(), e);
    }

    pub fn add_admin(&mut self, e: TeamUid, c: UserUid) -> Result<(), Error> {
        match self.teams.get_mut(&e.clone().into()) {
            Some(t) => {
                t.add_admin(c);
                Ok(())
            }
            None => Err(Error::no_such_entity(e)),
        }
    }

    pub fn remove_admin(&mut self, e: TeamUid, c: UserUid) -> Result<(), Error> {
        match self.teams.get_mut(&e.clone().into()) {
            Some(t) => {
                t.remove_admin(c);
                Ok(())
            }
            None => Err(Error::no_such_entity(e)),
        }
    }

    pub fn delete_entity(&mut self, e: impl AsRef<EntityUid>) -> Result<(), Error> {
        let r = e.as_ref();
        if self.users.contains_key(r) {
            self.users.remove(r);
            Ok(())
        } else if self.teams.contains_key(r) {
            self.teams.remove(r);
            Ok(())
        } else if self.lists.contains_key(r) {
            self.lists.remove(r);
            Ok(())
        } else {
            Err(Error::NoSuchEntity(r.clone()))
        }
    }

    pub fn get_user(&self, euid: &UserUid) -> Result<&User, Error> {
        self.users
            .get(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn get_user_mut(&mut self, euid: &UserUid) -> Result<&mut User, Error> {
        self.users
            .get_mut(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn get_team(&self, euid: &TeamUid) -> Result<&Team, Error> {
        self.teams
            .get(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn get_team_mut(&mut self, euid: &TeamUid) -> Result<&mut Team, Error> {
        self.teams
            .get_mut(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn get_user_or_team_mut(
        &mut self,
        euid: &UserOrTeamUid,
    ) -> Result<&mut dyn UserOrTeam, Error> {
        let euid_ref = euid.as_ref();
        if self.users.contains_key(euid_ref) {
            let u = self.users.get_mut(euid_ref).unwrap();
            Ok(u)
        } else if self.teams.contains_key(euid_ref) {
            let t = self.teams.get_mut(euid_ref).unwrap();
            Ok(t)
        } else {
            Err(Error::no_such_entity(euid_ref.clone()))
        }
    }

    pub fn get_list(&self, euid: &ListUid) -> Result<&List, Error> {
        self.lists
            .get(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn get_list_mut(&mut self, euid: &ListUid) -> Result<&mut List, Error> {
        self.lists
            .get_mut(euid.as_ref())
            .ok_or_else(|| Error::no_such_entity(euid.clone()))
    }

    pub fn add_user_to_team(&mut self, candidate: UserUid, team: TeamUid) -> Result<(), Error> {
        // TODO: `get_team` and `get_mut` should trivially succeed after
        // successful authorization
        let _ = self.get_team(&team)?;
        match self.users.get_mut(&candidate.clone().into()) {
            Some(user) => {
                user.insert_parent(team);
                Ok(())
            }
            None => Err(Error::no_such_entity(candidate)),
        }
    }

    pub fn remove_user_from_team(
        &mut self,
        candidate: UserUid,
        team: TeamUid,
    ) -> Result<(), Error> {
        // TODO: `get_team` and `get_mut` should trivially succeed after
        // successful authorization
        let _ = self.get_team(&team)?;
        match self.users.get_mut(&candidate.clone().into()) {
            Some(user) => {
                user.delete_parent(&team);
                Ok(())
            }
            None => Err(Error::no_such_entity(candidate)),
        }
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
