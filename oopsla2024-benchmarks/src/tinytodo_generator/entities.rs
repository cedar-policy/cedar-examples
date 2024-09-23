use std::collections::HashMap;

use arbitrary::{Arbitrary, Unstructured};
use lazy_static::lazy_static;
use serde::Serialize;

use super::{
    common::{construct_mapping, Entity, HasId, Uid},
    constants::NUM_USERS,
    list::{arbitrary_lists, List},
    teams::generate_teams,
    users::arbitrary_users,
};

#[derive(Debug, Clone, Serialize)]
pub struct Entities {
    pub users: HashMap<Uid, Entity>,
    pub teams: HashMap<Uid, Entity>,
    pub lists: HashMap<Uid, List>,
    app: App,
}

lazy_static! {
    pub static ref ACTIONS: Vec<cedar_policy_core::ast::Entity> = vec![
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"CreateList""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"GetLists""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"GetList""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"UpdateList""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"DeleteList""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"CreateTask""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"UpdateTask""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"DeleteTask""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Action::"EditShares""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
        cedar_policy_core::ast::Entity::new_with_attr_partial_value(
            r#"Application::"TinyTodo""#.parse().unwrap(),
            Default::default(),
            Default::default()
        ),
    ];
}

impl Entities {
    pub fn to_cedar_entities(&self) -> Vec<cedar_policy_core::ast::Entity> {
        let users = self.users.values().map(|user| user.to_cedar_entity());
        let teams = self.teams.values().map(|team| team.to_cedar_entity());
        let lists = self.lists.values().map(|list| list.to_cedar_entity());
        users
            .chain(teams)
            .chain(lists)
            .chain((*ACTIONS).iter().cloned())
            .collect()
    }

    pub fn arbitrary_user(&self, u: &mut Unstructured<'_>) -> arbitrary::Result<Uid> {
        let users: Vec<_> = self.users.keys().collect();
        u.choose(&users).map(|u| (*u).clone())
    }
    pub fn arbitrary_list(&self, u: &mut Unstructured<'_>) -> arbitrary::Result<Uid> {
        let lists: Vec<_> = self.lists.keys().collect();
        u.choose(&lists).map(|u| (*u).clone())
    }

    pub fn users(&self) -> impl Iterator<Item = &Entity> {
        self.users.values()
    }

    pub fn teams(&self) -> impl Iterator<Item = &Entity> {
        self.teams.values()
    }

    pub fn list(&self, uid: &Uid) -> Option<&List> {
        self.lists.get(uid)
    }

    pub fn users_in_team<'a>(&'a self, team: &'a Uid) -> impl Iterator<Item = &'a Entity> + 'a {
        self.users
            .values()
            .filter(|user| user.parents.contains(team))
    }

    pub fn num_entities(&self) -> usize {
        let u = self.users.len() + self.teams.len() + self.lists.len() + 1;
        eprintln!("Total entities: {u}");
        u
    }

    pub fn from_cedar_entities(
        entities: impl IntoIterator<Item = cedar_policy_core::ast::Entity>,
    ) -> Self {
        let user_entity_type = cedar_policy_core::ast::Name::parse_unqualified_name("User")
            .unwrap()
            .into();
        let team_entity_type = cedar_policy_core::ast::Name::parse_unqualified_name("Team")
            .unwrap()
            .into();
        let list_entity_type = cedar_policy_core::ast::Name::parse_unqualified_name("List")
            .unwrap()
            .into();
        let mut users = HashMap::new();
        let mut teams = HashMap::new();
        let mut lists = HashMap::new();
        for entity in entities {
            if entity.uid().entity_type() == &user_entity_type {
                users.insert(entity.uid().into(), Entity::from_cedar_entity(entity));
            } else if entity.uid().entity_type() == &team_entity_type {
                teams.insert(entity.uid().into(), Entity::from_cedar_entity(entity));
            } else if entity.uid().entity_type() == &list_entity_type {
                lists.insert(entity.uid().into(), List::from_cedar_entity(entity));
            }
        }
        Self {
            users,
            teams,
            lists,
            app: App::default(),
        }
    }
}

impl<'a> Arbitrary<'a> for Entities {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut teams = NUM_USERS.with(|num_users| generate_teams(u, *num_users.borrow()))?;
        let team_uids: Vec<_> = teams.iter().map(|e| e.id()).collect();
        let users =
            NUM_USERS.with(|num_users| arbitrary_users(u, &team_uids, *num_users.borrow()))?;
        let user_uids: Vec<_> = users.iter().map(|e| e.id()).collect();
        let (lists, mut list_teams) = arbitrary_lists(u, &user_uids)?;
        teams.append(&mut list_teams);
        drop(list_teams);

        Ok(Entities {
            users: construct_mapping(users),
            teams: construct_mapping(teams),
            lists: construct_mapping(lists),
            app: App::default(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct App {
    euid: &'static str,
}

impl Default for App {
    fn default() -> Self {
        Self {
            euid: r#"Application::"TinyTodo""#,
        }
    }
}
