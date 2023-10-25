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

use std::collections::{HashMap, HashSet};

use cedar_policy::{Entity, EvalResult, RestrictedExpression};
use serde::{Deserialize, Serialize};

use crate::{
    api::ShareRole,
    context::APPLICATION_TINY_TODO,
    entitystore::{EntityDecodeError, EntityStore},
    util::{EntityUid, ListUid, TeamUid, UserUid, TYPE_TEAM},
};

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
        Entity::new(
            a.euid().clone().into(),
            HashMap::default(),
            HashSet::default(),
        )
    }
}

pub trait UserOrTeam {
    fn insert_parent(&mut self, parent: TeamUid);
    fn delete_parent(&mut self, parent: &TeamUid);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    euid: UserUid,
    parents: HashSet<EntityUid>,
}

impl User {
    pub fn uid(&self) -> &UserUid {
        &self.euid
    }

    pub fn new(euid: UserUid) -> Self {
        let parent = Application::default().euid().clone();
        Self {
            euid,
            parents: [parent].into_iter().collect(),
        }
    }
}

impl From<User> for Entity {
    fn from(value: User) -> Entity {
        let euid: EntityUid = value.euid.into();
        Entity::new(
            euid.into(),
            HashMap::new(),
            value.parents.into_iter().map(|euid| euid.into()).collect(),
        )
    }
}

impl UserOrTeam for User {
    fn insert_parent(&mut self, parent: TeamUid) {
        self.parents.insert(parent.into());
    }

    fn delete_parent(&mut self, parent: &TeamUid) {
        self.parents.remove(parent.as_ref());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    uid: TeamUid,
    parents: HashSet<EntityUid>,
}

impl Team {
    pub fn new(euid: TeamUid) -> Team {
        let parent = Application::default().euid().clone();
        Self {
            uid: euid,
            parents: [parent].into_iter().collect(),
        }
    }

    pub fn uid(&self) -> &TeamUid {
        &self.uid
    }
}

impl From<Team> for Entity {
    fn from(team: Team) -> Entity {
        let euid: EntityUid = team.uid.into();
        Entity::new(
            euid.into(),
            HashMap::default(),
            team.parents.into_iter().map(|euid| euid.into()).collect(),
        )
    }
}

impl UserOrTeam for Team {
    fn insert_parent(&mut self, parent: TeamUid) {
        self.parents.insert(parent.into());
    }

    fn delete_parent(&mut self, parent: &TeamUid) {
        self.parents.remove(parent.as_ref());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    uid: ListUid,
    owner: UserUid,
    name: String,
    tasks: Vec<Task>, // Invariant, `tasks` must be sorted
    readers: TeamUid,
    editors: TeamUid,
}

impl List {
    pub fn new(store: &mut EntityStore, uid: ListUid, owner: UserUid, name: String) -> Self {
        let readers_uid = store.fresh_euid::<TeamUid>(TYPE_TEAM.clone()).unwrap();
        let readers = Team::new(readers_uid.clone());
        let writers_uid = store.fresh_euid::<TeamUid>(TYPE_TEAM.clone()).unwrap();
        let writers = Team::new(writers_uid.clone());
        store.insert_team(readers);
        store.insert_team(writers);
        Self {
            uid,
            owner,
            name,
            tasks: vec![],
            readers: readers_uid,
            editors: writers_uid,
        }
    }

    pub fn uid(&self) -> &ListUid {
        &self.uid
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

    pub fn get_team(&self, role: ShareRole) -> &TeamUid {
        match role {
            ShareRole::Reader => &self.readers,
            ShareRole::Editor => &self.editors,
        }
    }
}

impl From<List> for Entity {
    fn from(value: List) -> Self {
        let attrs = [
            (
                "owner",
                format!("{}", value.owner.as_ref()).parse().unwrap(),
            ),
            ("name", RestrictedExpression::new_string(value.name)),
            (
                "tasks",
                RestrictedExpression::new_set(value.tasks.into_iter().map(|t| t.into())),
            ),
            (
                "readers",
                format!("{}", value.readers.as_ref()).parse().unwrap(),
            ),
            (
                "editors",
                format!("{}", value.editors.as_ref()).parse().unwrap(),
            ),
        ]
        .into_iter()
        .map(|(x, v)| (x.into(), v))
        .collect();

        // We always have the single parent of the application, so we just hard code that here
        let parents = [APPLICATION_TINY_TODO.clone().into()]
            .into_iter()
            .collect::<HashSet<_>>();

        let euid: EntityUid = value.uid.into();
        Entity::new(euid.into(), attrs, parents)
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
        .map(|(x, v)| (x.to_string(), v))
        .expect("no duplicate keys!");
        RestrictedExpression::new_record(fields)
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
