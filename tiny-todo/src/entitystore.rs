use std::collections::HashMap;
use thiserror::Error;

use cedar_policy::{Entities, Entity, EntityId, EntityTypeName, EvaluationError};
use serde::{Deserialize, Serialize};

use crate::{
    context::Error,
    objects::{Application, List, Team, User, UserOrTeam},
    util::EntityUid,
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

    pub fn as_entities(&self) -> Entities {
        //Entities::from_entities(self.store.values().map(Entity::as_entity)).unwrap()
        let users = self.users.values().map(|user| user.pack());
        let teams = self.teams.values().map(|team| team.pack());
        let lists = self.lists.values().map(|list| list.pack());
        let app = std::iter::once(self.app.pack());
        let all = users.chain(teams).chain(lists).chain(app);
        Entities::from_entities(all).unwrap()
    }

    // Realistically you'd want to use something like a UUID here
    pub fn fresh_euid(&mut self, ty: EntityTypeName) -> EntityUid {
        loop {
            let new_uid: EntityId = format!("{}", self.uid).parse().unwrap();
            self.uid += 1;
            let euid = cedar_policy::EntityUid::from_type_name_and_id(ty.clone(), new_uid).into();
            if !self.euid_exists(&euid) {
                return euid;
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
        self.users.insert(e.uid().clone(), e);
    }

    pub fn insert_team(&mut self, e: Team) {
        self.teams.insert(e.uid().clone(), e);
    }

    pub fn insert_list(&mut self, e: List) {
        self.lists.insert(e.uid().clone(), e);
    }

    pub fn delete_entity(&mut self, e: &EntityUid) -> Result<(), Error> {
        //     .ok_or_else(|| Error::NoSuchEntity(e.clone()))
        if self.users.contains_key(e) {
            self.users.remove(e);
            Ok(())
        } else if self.teams.contains_key(e) {
            self.teams.remove(e);
            Ok(())
        } else if self.lists.contains_key(e) {
            self.lists.remove(e);
            Ok(())
        } else {
            Err(Error::NoSuchEntity(e.clone()))
        }
    }

    pub fn get_user(&self, euid: &EntityUid) -> Result<&User, Error> {
        self.users
            .get(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_user_mut(&mut self, euid: &EntityUid) -> Result<&mut User, Error> {
        self.users
            .get_mut(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_team(&self, euid: &EntityUid) -> Result<&Team, Error> {
        self.teams
            .get(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_team_mut(&mut self, euid: &EntityUid) -> Result<&mut Team, Error> {
        self.teams
            .get_mut(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_user_or_team_mut(&mut self, euid: &EntityUid) -> Result<&mut dyn UserOrTeam, Error> {
        if self.users.contains_key(euid) {
            let u = self.users.get_mut(euid).unwrap();
            Ok(u)
        } else if self.teams.contains_key(euid) {
            let t = self.teams.get_mut(euid).unwrap();
            Ok(t)
        } else {
            Err(Error::NoSuchEntity(euid.clone()))
        }
    }

    pub fn get_list(&self, euid: &EntityUid) -> Result<&List, Error> {
        self.lists
            .get(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_list_mut(&mut self, euid: &EntityUid) -> Result<&mut List, Error> {
        self.lists
            .get_mut(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }
}

pub trait TypedEntity: Sized {
    //fn unpack(e: &'a Entity) -> Result<Self, EntityDecodeError>;
    fn pack(&self) -> Entity;
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
