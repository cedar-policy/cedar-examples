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

use std::{ops::Deref, str::FromStr};

use cedar_policy::{EntityTypeName, ParseErrors, RestrictedExpression};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Expression(
    #[serde(deserialize_with = "deserialize_restricted_expression")] RestrictedExpression,
);

impl From<RestrictedExpression> for Expression {
    fn from(value: RestrictedExpression) -> Self {
        Self(value)
    }
}

impl From<Expression> for RestrictedExpression {
    fn from(value: Expression) -> Self {
        value.0
    }
}

impl Deref for Expression {
    type Target = RestrictedExpression;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn deserialize_restricted_expression<'de, D>(d: D) -> Result<RestrictedExpression, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'ide> serde::de::Visitor<'ide> for Visitor {
        type Value = RestrictedExpression;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "Expected string that could be parsed as a Restricted Cedar Expression"
            )
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid: RestrictedExpression = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid: RestrictedExpression = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }

        fn visit_borrowed_str<E>(self, v: &'ide str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid: RestrictedExpression = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }
    }

    d.deserialize_str(Visitor)
}

lazy_static! {
    pub static ref TYPE_LIST: EntityTypeName = "List".parse().unwrap();
    pub static ref TYPE_USER: EntityTypeName = "User".parse().unwrap();
    pub static ref TYPE_TEAM: EntityTypeName = "Team".parse().unwrap();
}

// Here we defined a bunch of typed wrappers around `EntityUid`.
// This lets us ensure that if we have a value of type `ListUid`,
// we know we have `EntityUid` with type `List`.
// Because these are single-value struct wrappers, it is free to convert between them.

#[derive(Debug, Clone, Error)]
pub struct EntityTypeError {
    // Non empty vec of types
    expected: (&'static EntityTypeName, Vec<&'static EntityTypeName>),
    got: EntityUid,
}

impl EntityTypeError {
    pub fn single(expected: &'static EntityTypeName, got: EntityUid) -> Self {
        Self {
            expected: (expected, vec![]),
            got,
        }
    }

    pub fn multiple(
        first: &'static EntityTypeName,
        rest: Vec<&'static EntityTypeName>,
        got: EntityUid,
    ) -> Self {
        Self {
            expected: (first, rest),
            got,
        }
    }
}

impl std::fmt::Display for EntityTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut expected = self.expected.1.iter().collect::<Vec<_>>();
        if expected.is_empty() {
            write!(
                f,
                "Expected an entity of type {}, got: {}",
                self.expected.0, self.got
            )
        } else {
            expected.push(&self.expected.0);
            write!(
                f,
                "Expected one of the following entity types: {}, got: {}",
                expected.into_iter().join(","),
                self.got
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "EntityUid")]
#[serde(into = "EntityUid")]
#[repr(transparent)]
pub struct UserUid(EntityUid);

impl TryFrom<EntityUid> for UserUid {
    type Error = EntityTypeError;
    fn try_from(got: EntityUid) -> Result<Self, Self::Error> {
        entity_type_check(&TYPE_USER, got, Self)
    }
}

impl From<UserUid> for EntityUid {
    fn from(value: UserUid) -> Self {
        value.0
    }
}

impl AsRef<EntityUid> for UserUid {
    fn as_ref(&self) -> &EntityUid {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "EntityUid")]
#[serde(into = "EntityUid")]
#[repr(transparent)]
pub struct ListUid(EntityUid);

impl TryFrom<EntityUid> for ListUid {
    type Error = EntityTypeError;
    fn try_from(got: EntityUid) -> Result<Self, Self::Error> {
        entity_type_check(&TYPE_LIST, got, Self)
    }
}

impl From<ListUid> for EntityUid {
    fn from(l: ListUid) -> Self {
        l.0
    }
}

impl AsRef<EntityUid> for ListUid {
    fn as_ref(&self) -> &EntityUid {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "EntityUid")]
#[serde(into = "EntityUid")]
#[repr(transparent)]
pub struct UserOrTeamUid(EntityUid);

impl TryFrom<EntityUid> for UserOrTeamUid {
    type Error = EntityTypeError;
    fn try_from(got: EntityUid) -> Result<Self, Self::Error> {
        let r: Result<UserUid, Self::Error> = got.clone().try_into();
        if let Ok(user) = r {
            Ok(user.into())
        } else {
            let r: Result<TeamUid, Self::Error> = got.clone().try_into();
            if let Ok(team) = r {
                Ok(team.into())
            } else {
                Err(EntityTypeError::multiple(&TYPE_USER, vec![&TYPE_TEAM], got))
            }
        }
    }
}

impl AsRef<EntityUid> for UserOrTeamUid {
    fn as_ref(&self) -> &EntityUid {
        &self.0
    }
}

impl From<UserUid> for UserOrTeamUid {
    fn from(value: UserUid) -> Self {
        Self(value.0)
    }
}

impl From<TeamUid> for UserOrTeamUid {
    fn from(value: TeamUid) -> Self {
        Self(value.0)
    }
}

impl From<UserOrTeamUid> for EntityUid {
    fn from(l: UserOrTeamUid) -> Self {
        l.0
    }
}

impl From<UserUid> for RestrictedExpression {
    fn from(value: UserUid) -> Self {
        RestrictedExpression::new_entity_uid(value.as_ref().clone().into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "EntityUid")]
#[serde(into = "EntityUid")]
#[repr(transparent)]
pub struct TeamUid(EntityUid);

impl TryFrom<EntityUid> for TeamUid {
    type Error = EntityTypeError;
    fn try_from(got: EntityUid) -> Result<Self, Self::Error> {
        entity_type_check(&TYPE_TEAM, got, Self)
    }
}

impl From<TeamUid> for RestrictedExpression {
    fn from(value: TeamUid) -> Self {
        RestrictedExpression::new_entity_uid(value.as_ref().clone().into())
    }
}

impl AsRef<EntityUid> for TeamUid {
    fn as_ref(&self) -> &EntityUid {
        &self.0
    }
}

impl From<TeamUid> for EntityUid {
    fn from(value: TeamUid) -> Self {
        value.0
    }
}

fn entity_type_check<T>(
    expected: &'static EntityTypeName,
    got: EntityUid,
    f: impl FnOnce(EntityUid) -> T,
) -> Result<T, EntityTypeError> {
    if expected == got.0.type_name() {
        Ok(f(got))
    } else {
        Err(EntityTypeError::single(expected, got))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct EntityUid(
    #[serde(serialize_with = "serialize_euid")]
    #[serde(deserialize_with = "deserialize_euid")]
    cedar_policy::EntityUid,
);

impl AsRef<EntityUid> for EntityUid {
    fn as_ref(&self) -> &EntityUid {
        self
    }
}

impl FromStr for EntityUid {
    type Err = ParseErrors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e: cedar_policy::EntityUid = s.parse()?;
        Ok(e.into())
    }
}

impl std::fmt::Display for EntityUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<cedar_policy::EntityUid> for EntityUid {
    fn from(value: cedar_policy::EntityUid) -> Self {
        Self(value)
    }
}

impl Deref for EntityUid {
    type Target = cedar_policy::EntityUid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<EntityUid> for cedar_policy::EntityUid {
    fn from(value: EntityUid) -> Self {
        value.0
    }
}

pub fn serialize_euid<S>(euid: &cedar_policy::EntityUid, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{euid}"))
}

pub fn deserialize_euid<'de, D>(d: D) -> Result<cedar_policy::EntityUid, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'ide> serde::de::Visitor<'ide> for Visitor {
        type Value = cedar_policy::EntityUid;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "Expected string that could be parsed as an EntityUid"
            )
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }

        fn visit_borrowed_str<E>(self, v: &'ide str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let euid = v
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
            Ok(euid)
        }
    }

    d.deserialize_str(Visitor)
}
