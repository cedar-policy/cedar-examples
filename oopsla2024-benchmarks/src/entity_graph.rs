use cedar_policy_core::{
    entities::{Entities, NoEntitiesSchema},
    extensions::Extensions,
};
use cedar_policy_generators::collections::HashMap;

pub trait EntityGraph {
    fn from_iter<'a>(i: impl IntoIterator<Item = &'a cedar_policy_core::ast::Entity>) -> Self;
    fn iter(&self) -> Box<dyn Iterator<Item = cedar_policy_core::ast::Entity>>;
    fn get<'a>(&'a self, x: &str) -> Option<&'a cedar_policy_core::ast::Entity>;
}

impl EntityGraph for Entities {
    fn from_iter<'a>(i: impl IntoIterator<Item = &'a cedar_policy_core::ast::Entity>) -> Self {
        Entities::from_entities(
            i.into_iter().cloned(),
            None::<&NoEntitiesSchema>,
            cedar_policy_core::entities::TCComputation::ComputeNow,
            Extensions::all_available(),
        )
        .unwrap()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = cedar_policy_core::ast::Entity>> {
        let v: Vec<_> = self.iter().cloned().collect();
        Box::new(v.into_iter())
    }

    fn get<'a>(&'a self, x: &str) -> Option<&'a cedar_policy_core::ast::Entity> {
        let euid = x.parse().unwrap();
        match self.entity(&euid) {
            cedar_policy_core::entities::Dereference::Data(e) => Some(e),
            _ => None,
        }
    }
}

pub struct OpenEntities {
    entities: HashMap<String, cedar_policy_core::ast::Entity>,
}

impl EntityGraph for OpenEntities {
    fn from_iter<'a>(i: impl IntoIterator<Item = &'a cedar_policy_core::ast::Entity>) -> Self {
        let entities = i
            .into_iter()
            .map(|e| (e.uid().to_string(), e.clone()))
            .collect();
        Self { entities }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = cedar_policy_core::ast::Entity>> {
        let v: Vec<_> = self
            .entities
            .values()
            .map(|entity| (*entity).clone())
            .collect();
        Box::new(v.into_iter())
    }

    fn get<'a>(&'a self, x: &str) -> Option<&'a cedar_policy_core::ast::Entity> {
        self.entities.get(x)
    }
}
