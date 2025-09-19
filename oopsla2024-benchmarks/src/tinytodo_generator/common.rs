use std::collections::{BTreeMap, HashMap, HashSet};

use arbitrary::Unstructured;
use serde::Serialize;

pub trait HasId {
    fn id(&self) -> Uid;
}

#[derive(Debug, Clone, Serialize)]
pub struct Entity {
    pub euid: Uid,
    pub parents: Vec<Uid>,
}

impl Entity {
    pub fn to_cedar_entity(&self) -> cedar_policy_core::ast::Entity {
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            self.euid.to_euid(),
            HashMap::new(),
            HashSet::new(),
            self.parents.iter().map(|uid| uid.to_euid()).collect(),
            BTreeMap::new(),
        )
    }

    pub fn from_cedar_entity(e: cedar_policy_core::ast::Entity) -> Self {
        Self {
            euid: e.uid().into(),
            parents: e.ancestors().map(Into::into).collect(),
        }
    }
}

impl HasId for Entity {
    fn id(&self) -> Uid {
        self.euid.clone()
    }
}

#[derive(Debug, Clone, Serialize, Ord, PartialEq, Eq, PartialOrd, Hash)]
#[serde(transparent)]
pub struct Uid(String);

impl Uid {
    pub fn to_euid(&self) -> cedar_policy_core::ast::EntityUID {
        self.0.parse().unwrap()
    }

    pub fn is_user(&self) -> bool {
        self.0.contains("User")
    }
}

impl From<cedar_policy_core::ast::EntityUID> for Uid {
    fn from(uid: cedar_policy_core::ast::EntityUID) -> Self {
        Self::from(&uid)
    }
}

impl<'a> From<&'a cedar_policy_core::ast::EntityUID> for Uid {
    fn from(uid: &'a cedar_policy_core::ast::EntityUID) -> Self {
        Uid(uid.to_string())
    }
}

pub fn application() -> Uid {
    Uid(r#"Application::"TinyTodo""#.to_string())
}

pub fn construct_mapping<E: HasId>(entities: impl IntoIterator<Item = E>) -> HashMap<Uid, E> {
    entities.into_iter().map(|e| (e.id(), e)).collect()
}

pub fn make_uid(group: &str, id: &str) -> Uid {
    Uid(format!("{group}::\"{id}\""))
}

#[derive(Default)]
pub struct NameGenerator {
    contents: HashSet<String>,
}

const NAME_LENGTH: usize = 5;
const BOUND: usize = 50;

impl NameGenerator {
    pub fn fresh(&mut self, u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
        for _ in 0..BOUND {
            let name = Self::alphaname(u)?;
            if !(self.contents.contains(&name)) {
                self.contents.insert(name.to_string());
                return Ok(name);
            }
        }
        Err(arbitrary::Error::NotEnoughData)
    }

    fn alphaname(u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
        let mut chars = vec![];
        for _ in 0..NAME_LENGTH {
            let a: u8 = u.int_in_range(65..=90)?;
            chars.push(a);
        }
        Ok(String::from_utf8(chars).unwrap())
    }
}

pub fn choose<'a, T>(u: &mut Unstructured<'_>, t: &'a [T]) -> Result<&'a T, arbitrary::Error> {
    match u.choose(t) {
        Ok(a) => Ok(a),
        Err(arbitrary::Error::EmptyChoose) => panic!("Empty Choose"),
        Err(e) => Err(e),
    }
}
