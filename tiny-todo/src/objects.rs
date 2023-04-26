use std::collections::HashSet;

use cedar::{EntityTypeName, EvalResult, RestrictedExpression};
use serde::{Deserialize, Serialize};

use crate::{
    api::ShareRole,
    entitystore::{Entity, EntityDecodeError, EntityStore, TypedEntity},
    util::EntityUid,
};

#[derive(Debug)]
pub struct Team {
    uid: EntityUid,
    name: Option<String>,
    owner: EntityUid,
}

impl Team {
    pub fn new(euid: EntityUid, name: Option<String>, owner: EntityUid) -> Team {
        Self {
            uid: euid,
            name,
            owner,
        }
    }
}

impl From<Team> for Entity {
    fn from(team: Team) -> Entity {
        let owner_tup = (
            "owner".to_string(),
            format!("{}", team.owner).parse().unwrap(),
        );
        let attrs = if let Some(name) = team.name {
            [
                owner_tup,
                ("name".to_string(), RestrictedExpression::new_string(name)),
            ]
            .into_iter()
            .collect()
        } else {
            [owner_tup].into_iter().collect()
        };

        let parents = HashSet::new();
        Entity::new(team.uid, attrs, parents)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct List {
    uid: EntityUid,
    owner: EntityUid,
    name: String,
    // Invariant, `tasks` must be sorted
    tasks: Vec<Task>,

    readers: EntityUid,
    editors: EntityUid,
}

impl<'a> TypedEntity<'a> for List {
    fn unpack(entity: &'a Entity) -> Result<Self, EntityDecodeError> {
        let owner_field = "owner";
        let tasks_field = "tasks";
        let name_field = "name";

        let owner = match entity
            .attr(owner_field)
            .ok_or(EntityDecodeError::MissingAttr(owner_field))??
        {
            EvalResult::EntityUid(euid) => Ok(euid),
            _ => Err(EntityDecodeError::WrongType(owner_field, "EntityUid")),
        }?;

        let name = get_string(
            &entity
                .attr(name_field)
                .ok_or(EntityDecodeError::MissingAttr(name_field))??,
            name_field,
        )?
        .clone();

        let readers_field = "readers";
        let editors_field = "editors";
        let readers = decode_euid(
            &entity
                .attr(readers_field)
                .ok_or(EntityDecodeError::MissingAttr(readers_field))??,
            readers_field,
        )?;
        let editors = decode_euid(
            &entity
                .attr(editors_field)
                .ok_or(EntityDecodeError::MissingAttr(editors_field))??,
            editors_field,
        )?;

        let mut tasks = match entity
            .attr(tasks_field)
            .ok_or(EntityDecodeError::MissingAttr(tasks_field))??
        {
            cedar::EvalResult::Set(tasks) => Ok(tasks
                .iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<Task>, _>>()?),
            _ => Err(EntityDecodeError::WrongType(tasks_field, "Set")),
        }?;

        tasks.sort();
        Ok(Self {
            name,
            uid: entity.uid(),
            owner: owner.into(),
            tasks,
            readers,
            editors,
        })
    }
}

fn decode_euid(e: &EvalResult, field: &'static str) -> Result<EntityUid, EntityDecodeError> {
    match e {
        EvalResult::EntityUid(euid) => Ok(euid.clone().into()),
        _ => Err(EntityDecodeError::WrongType(field, "Set of entity uids")),
    }
}

impl List {
    pub fn new(store: &mut EntityStore, uid: EntityUid, owner: EntityUid, name: String) -> Self {
        let team_ty: EntityTypeName = "Team".parse().unwrap();
        let readers_uid = store.fresh_euid(team_ty.clone());
        let readers = Team::new(readers_uid.clone(), None, uid.clone()).into();
        let writers_uid = store.fresh_euid(team_ty);
        let writers = Team::new(writers_uid.clone(), None, uid.clone()).into();
        store.insert_entity(readers);
        store.insert_entity(writers);
        Self {
            uid,
            owner,
            name,
            tasks: vec![],
            readers: readers_uid,
            editors: writers_uid,
        }
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

    pub fn get_team(&self, role: ShareRole) -> &EntityUid {
        match role {
            ShareRole::Reader => &self.readers,
            ShareRole::Editor => &self.editors,
        }
    }
}

impl From<List> for Entity {
    fn from(value: List) -> Self {
        let attrs = [
            ("owner", format!("{}", value.owner).parse().unwrap()),
            ("name", RestrictedExpression::new_string(value.name)),
            (
                "tasks",
                RestrictedExpression::new_set(value.tasks.into_iter().map(|t| t.into())),
            ),
            ("readers", format!("{}", value.readers).parse().unwrap()),
            ("editors", format!("{}", value.editors).parse().unwrap()),
        ]
        .into_iter()
        .map(|(x, v)| (x.into(), v))
        .collect();
        Entity::new(value.uid, attrs, HashSet::default())
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
