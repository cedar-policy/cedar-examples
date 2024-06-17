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

use std::collections::HashSet;

use cedar_policy::{Entity, EvalResult, RestrictedExpression};
use serde::{Deserialize, Serialize};

use crate::{
    context::APPLICATION_TINY_TODO,
    entitystore::{EntityDecodeError, EntityStore},
    util::{make_list_euid, make_team_euid, make_user_euid, EntityUid, UserOrTeamUid},
};

#[cfg(not(feature = "use-templates"))]
use crate::api::ShareRole;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Application {
    euid: EntityUid,
}

impl Application {
    pub fn euid(&self) -> &EntityUid {
        &self.euid
    }
}

impl Default for Application {
    fn default() -> Self {
        Application {
            euid: APPLICATION_TINY_TODO.clone(),
        }
    }
}

impl From<Application> for Entity {
    fn from(a: Application) -> Self {
        Entity::new_no_attrs(a.euid().clone().into(), HashSet::default())
    }
}

pub trait UserOrTeam {
    fn insert_parent(&mut self, parent: &UserOrTeamUid);
    fn delete_parent(&mut self, parent: &UserOrTeamUid);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    eid: String,
    joblevel: i64,
    location: String,
    parents: HashSet<EntityUid>,
}

impl User {
    pub fn eid(&self) -> String {
        self.eid.to_owned()
    }

    pub fn new(eid: &str, joblevel: i64, location: String) -> Self {
        let parent = Application::default().euid().clone();
        Self {
            eid: eid.to_owned(),
            joblevel,
            location,
            parents: [parent].into_iter().collect(),
        }
    }
}

impl From<User> for Entity {
    fn from(value: User) -> Entity {
        let attrs = [
            ("joblevel", RestrictedExpression::new_long(value.joblevel)),
            ("location", RestrictedExpression::new_string(value.location)),
        ]
        .into_iter()
        .map(|(x, v)| (x.into(), v))
        .collect();

        let euid: EntityUid = make_user_euid(&value.eid);
        Entity::new(
            euid.into(),
            attrs,
            value.parents.into_iter().map(|euid| euid.into()).collect(),
        )
        .unwrap()
    }
}

impl UserOrTeam for User {
    fn insert_parent(&mut self, parent: &UserOrTeamUid) {
        self.parents.insert(parent.as_ref().clone());
    }

    fn delete_parent(&mut self, parent: &UserOrTeamUid) {
        self.parents.remove(parent.as_ref());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    eid: String,
    parents: HashSet<EntityUid>,
}

impl Team {
    pub fn new(eid: &str) -> Team {
        let parent = Application::default().euid().clone();
        Self {
            eid: eid.to_owned(),
            parents: [parent].into_iter().collect(),
        }
    }

    pub fn eid(&self) -> String {
        self.eid.to_owned()
    }
}

impl From<Team> for Entity {
    fn from(team: Team) -> Entity {
        let euid: EntityUid = make_team_euid(&team.eid);
        Entity::new_no_attrs(
            euid.into(),
            team.parents.into_iter().map(|euid| euid.into()).collect(),
        )
    }
}

impl UserOrTeam for Team {
    fn insert_parent(&mut self, parent: &UserOrTeamUid) {
        self.parents.insert(parent.as_ref().clone());
    }

    fn delete_parent(&mut self, parent: &UserOrTeamUid) {
        self.parents.remove(parent.as_ref());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    eid: String,
    owner: String,
    name: String,
    tasks: Vec<Task>, // Invariant, `tasks` must be sorted
    #[cfg(not(feature = "use-templates"))]
    readers: String,
    #[cfg(not(feature = "use-templates"))]
    editors: String,
}

impl List {
    #![allow(unused_variables)]
    pub fn new(store: &mut EntityStore, eid: &str, owner: &str, name: String) -> Self {
        #[cfg(not(feature = "use-templates"))]
        {
            let readers_eid = store.fresh_eid();
            let readers = Team::new(&readers_eid);
            let writers_eid = store.fresh_eid();
            let writers = Team::new(&writers_eid);
            store.insert_team(readers);
            store.insert_team(writers);
            Self {
                eid: eid.to_owned(),
                owner: owner.to_owned(),
                name,
                tasks: vec![],
                readers: readers_eid,
                editors: writers_eid,
            }
        }
        #[cfg(feature = "use-templates")]
        Self {
            uid,
            owner,
            name,
            tasks: vec![],
        }
    }

    pub fn eid(&self) -> String {
        self.eid.clone()
    }

    pub fn create_task(&mut self, description: String) -> i64 {
        let id = self.tasks.len() as i64;
        let task = Task::new(id, description);
        self.tasks.push(task);
        id
    }

    pub fn get_task_mut(&mut self, id: i64) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    pub fn delete_task(&mut self, id: i64) -> Option<()> {
        for (indx, task) in self.tasks.iter().enumerate() {
            if task.id == id {
                self.tasks.remove(indx);
                return Some(());
            }
        }
        None
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    #[cfg(not(feature = "use-templates"))]
    pub fn get_team(&self, role: ShareRole) -> String {
        match role {
            ShareRole::Reader => self.readers.to_owned(),
            ShareRole::Editor => self.editors.to_owned(),
        }
    }
}

impl From<List> for Entity {
    fn from(value: List) -> Self {
        let attrs = [
            (
                "owner",
                format!("{}", make_user_euid(&value.owner)).parse().unwrap(),
            ),
            ("name", RestrictedExpression::new_string(value.name)),
            (
                "tasks",
                RestrictedExpression::new_set(value.tasks.into_iter().map(|t| t.into())),
            ),
            #[cfg(not(feature = "use-templates"))]
            (
                "readers",
                format!("{}", make_team_euid(&value.readers))
                    .parse()
                    .unwrap(),
            ),
            #[cfg(not(feature = "use-templates"))]
            (
                "editors",
                format!("{}", make_team_euid(&value.editors))
                    .parse()
                    .unwrap(),
            ),
        ]
        .into_iter()
        .map(|(x, v)| (x.into(), v))
        .collect();

        // We always have the single parent of the application, so we just hard code that here
        let parents = [APPLICATION_TINY_TODO.clone().into()]
            .into_iter()
            .collect::<HashSet<_>>();

        let euid: EntityUid = make_list_euid(&value.eid);
        Entity::new(euid.into(), attrs, parents).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: i64,
    name: String,
    state: TaskState,
}

impl Task {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            state: TaskState::Unchecked,
        }
    }

    pub fn set_name(&mut self, new: String) {
        self.name = new;
    }

    pub fn set_state(&mut self, new: TaskState) {
        self.state = new;
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl TryFrom<&EvalResult> for Task {
    type Error = EntityDecodeError;

    fn try_from(value: &EvalResult) -> Result<Self, Self::Error> {
        let id_field = "id";
        let name_field = "name";
        let state_field = "state";
        match value {
            EvalResult::Record(rcd) => {
                let id = get_long(
                    rcd.get(id_field)
                        .ok_or(EntityDecodeError::MissingAttr(id_field))?,
                    id_field,
                )?;
                let name = get_string(
                    rcd.get(name_field)
                        .ok_or(EntityDecodeError::MissingAttr(name_field))?,
                    name_field,
                )?
                .clone();
                let state = rcd
                    .get(state_field)
                    .ok_or(EntityDecodeError::MissingAttr(state_field))?
                    .try_into()?;
                Ok(Self { id, name, state })
            }
            _ => Err(EntityDecodeError::WrongType("task", "record")),
        }
    }
}

impl From<Task> for RestrictedExpression {
    fn from(value: Task) -> Self {
        let fields = [
            ("id", RestrictedExpression::new_long(value.id)),
            ("name", RestrictedExpression::new_string(value.name)),
            (
                "state",
                RestrictedExpression::new_string(format!("{}", value.state)),
            ),
        ]
        .into_iter()
        .map(|(x, v)| (x.to_string(), v));
        RestrictedExpression::new_record(fields).expect("no duplicate keys!")
    }
}

fn get_long(e: &EvalResult, name: &'static str) -> Result<i64, EntityDecodeError> {
    match e {
        EvalResult::Long(l) => Ok(*l),
        _ => Err(EntityDecodeError::WrongType(name, "Long")),
    }
}

fn get_string<'a>(e: &'a EvalResult, name: &'static str) -> Result<&'a String, EntityDecodeError> {
    match e {
        EvalResult::String(s) => Ok(s),
        _ => Err(EntityDecodeError::WrongType(name, "String")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Checked,
    Unchecked,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Checked => write!(f, "checked"),
            TaskState::Unchecked => write!(f, "unchecked"),
        }
    }
}

impl TryFrom<&EvalResult> for TaskState {
    type Error = EntityDecodeError;

    fn try_from(value: &EvalResult) -> Result<Self, Self::Error> {
        match value {
            EvalResult::String(s) => match s.as_str() {
                "checked" => Ok(TaskState::Checked),
                "unchecked" => Ok(TaskState::Unchecked),
                _ => Err(EntityDecodeError::BadEnum {
                    enumeration: "TaskState",
                    got: s.clone(),
                }),
            },
            _ => Err(EntityDecodeError::WrongType("state", "String")),
        }
    }
}
