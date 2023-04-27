use std::{ops::Deref, str::FromStr};

use cedar_policy::{ParseErrors, RestrictedExpression};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct EntityUid(
    #[serde(serialize_with = "serialize_euid")]
    #[serde(deserialize_with = "deserialize_euid")]
    cedar_policy::EntityUid,
);

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

#[derive(Debug, Clone, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Lists(Vec<EntityUid>);

impl From<Vec<EntityUid>> for Lists {
    fn from(value: Vec<EntityUid>) -> Self {
        Self(value)
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
