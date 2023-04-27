use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use thiserror::Error;

use cedar_policy::{
    Entities, EntityId, EntityTypeName, EvalResult, EvaluationError, RestrictedExpression,
};
use serde::Deserialize;

use crate::{
    context::Error,
    util::{EntityUid, Expression},
    APPLICATION,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Entity {
    uid: EntityUid,
    attrs: HashMap<String, Expression>,
    parents: HashSet<EntityUid>,
}

impl Entity {
    pub fn new(
        uid: EntityUid,
        attrs: HashMap<String, RestrictedExpression>,
        mut parents: HashSet<EntityUid>,
    ) -> Self {
        let app = APPLICATION.parse().unwrap();
        if app != uid {
            parents.insert(app);
        }
        Self {
            uid,
            attrs: attrs.into_iter().map(|(k, v)| (k, v.into())).collect(),
            parents,
        }
    }

    pub fn uid(&self) -> EntityUid {
        self.uid.clone()
    }

    pub fn as_entity(&self) -> cedar_policy::Entity {
        cedar_policy::Entity::new(
            self.uid.clone().into(),
            self.attrs
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v.deref().clone()))
                .collect(),
            self.parents.clone().into_iter().map(|x| x.into()).collect(),
        )
    }

    pub fn attr(&self, attr: &str) -> Option<Result<EvalResult, EvaluationError>> {
        cedar_policy::Entity::new(
            self.uid().into(),
            self.attrs
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            HashSet::new(),
        )
        .attr(attr)
    }

    pub fn add_parent(&mut self, p: EntityUid) {
        self.parents.insert(p);
    }

    pub fn remove_parent(&mut self, p: &EntityUid) {
        self.parents.remove(p);
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct EntityStore {
    store: HashMap<EntityUid, Entity>,
    uid: usize,
}

impl EntityStore {
    pub fn euids(&self) -> impl Iterator<Item = &EntityUid> {
        self.store.keys()
    }

    pub fn as_entities(&self) -> Entities {
        Entities::from_entities(self.store.values().map(Entity::as_entity)).unwrap()
    }

    // Realistically you'd want to use something like a UUID here
    pub fn fresh_euid(&mut self, ty: EntityTypeName) -> EntityUid {
        loop {
            let new_uid: EntityId = format!("{}", self.uid).parse().unwrap();
            self.uid += 1;
            let euid = cedar_policy::EntityUid::from_type_name_and_id(ty.clone(), new_uid).into();
            if !self.store.contains_key(&euid) {
                return euid;
            }
        }
    }

    pub fn insert_entity(&mut self, e: Entity) {
        self.store.insert(e.uid(), e);
    }

    pub fn delete_entity(&mut self, e: &EntityUid) -> Result<(), Error> {
        self.store
            .remove(e)
            .ok_or_else(|| Error::NoSuchEntity(e.clone()))
            .map(|_| ())
    }

    pub fn get(&self, euid: &EntityUid) -> Result<&Entity, Error> {
        self.store
            .get(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }

    pub fn get_mut(&mut self, euid: &EntityUid) -> Result<&mut Entity, Error> {
        self.store
            .get_mut(euid)
            .ok_or_else(|| Error::NoSuchEntity(euid.clone()))
    }
}

pub trait TypedEntity<'a>: Sized {
    fn unpack(e: &'a Entity) -> Result<Self, EntityDecodeError>;
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
