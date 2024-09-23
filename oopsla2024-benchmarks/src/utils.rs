#![allow(dead_code)] // this is a utilities file, some utilities are included for completeness even if currently unused

use cedar_policy_core::ast::{
    EntityType, EntityUID, Literal, Name, PartialValue, Value, ValueKind,
};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

/// Convert &str to EntityType. Assumes the EntityType is supposed to be
/// unqualified and the &str doesn't contain namespaces
pub fn entity_type(s: &str) -> EntityType {
    Name::parse_unqualified_name(s).unwrap().into()
}

/// Given a `PartialValue` that we expect to contain no unknowns,
/// get the `Value`, panicking if we find any unknowns
pub fn pv_expect_known(v: &PartialValue) -> &Value {
    match v {
        PartialValue::Value(v) => v,
        PartialValue::Residual(r) => panic!("expected a concrete value; got this residual: {r}"),
    }
}

/// Given a `Value` that we expect to be a boolean,
/// get the boolean value, panicking if it's not actually a boolean
pub fn expect_bool(v: &Value) -> bool {
    match &v.value {
        ValueKind::Lit(Literal::Bool(b)) => *b,
        v => panic!("expected a boolean; got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be a boolean,
/// get the boolean value, panicking if it's not actually a boolean (or is unknown)
pub fn pv_expect_bool(v: &PartialValue) -> bool {
    expect_bool(pv_expect_known(v))
}

/// Given a `Value` that we expect to be an int,
/// get the int value, panicking if it's not actually an int
pub fn expect_int(v: &Value) -> i64 {
    match &v.value {
        ValueKind::Lit(Literal::Long(i)) => *i,
        v => panic!("expected an int: got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be an int,
/// get the int value, panicking if it's not actually an int (or is unknown)
pub fn pv_expect_int(v: &PartialValue) -> i64 {
    expect_int(pv_expect_known(v))
}

/// Given a `Value` that we expect to be a string,
/// get the string value, panicking if it's not actually a string
pub fn expect_string(v: &Value) -> SmolStr {
    match &v.value {
        ValueKind::Lit(Literal::String(s)) => s.clone(),
        v => panic!("expected a string; got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be a string,
/// get the string value, panicking if it's not actually a string (or is unknown)
pub fn pv_expect_string(v: &PartialValue) -> SmolStr {
    expect_string(pv_expect_known(v))
}

/// Given a `Value` that we expect to be an EntityUID,
/// get the EntityUID value, panicking if it's not actually an EntityUID
pub fn expect_euid(v: &Value) -> Arc<EntityUID> {
    match &v.value {
        ValueKind::Lit(Literal::EntityUID(euid)) => Arc::clone(euid),
        v => panic!("expected an euid; got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be an EntityUID,
/// get the EntityUID value, panicking if it's not actually an EntityUID (or is unknown)
pub fn pv_expect_euid(v: &PartialValue) -> Arc<EntityUID> {
    expect_euid(pv_expect_known(v))
}

/// Given a `Value` that we expect to be a record,
/// get the map of (key, value) pairs in the record, panicking if it's not actually a record
pub fn expect_record(v: &Value) -> HashMap<SmolStr, &Value> {
    match &v.value {
        ValueKind::Record(record) => record.iter().map(|(k, v)| (k.clone(), v)).collect(),
        v => panic!("expected a record; got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be a record,
/// get the map of (key, value) pairs in the record, panicking if it's not actually a record (or is unknown)
pub fn pv_expect_record(v: &PartialValue) -> HashMap<SmolStr, &Value> {
    expect_record(pv_expect_known(v))
}

/// Given a `Value` that we expect to be a set,
/// iterate over the values in the set, panicking if it's not actually a set
pub fn expect_set<'a>(v: &'a Value) -> impl Iterator<Item = &'a Value> + 'a {
    match &v.value {
        ValueKind::Set(s) => s.iter(),
        v => panic!("expected a set; got: {v:?}"),
    }
}

/// Given an attribute value (`PartialValue`) that we expect to be a set,
/// iterate over the values in the set, panicking if it's not actually a set (or is unknown)
pub fn pv_expect_set<'a>(v: &'a PartialValue) -> impl Iterator<Item = &'a Value> + 'a {
    expect_set(pv_expect_known(v))
}

/// Given a `Value` that we expect to be a set of EUIDs,
/// iterate over those EUIDs, panicking if the `Value` isn't actually a set of EUIDs
pub fn expect_set_euids(v: &Value) -> impl Iterator<Item = Arc<EntityUID>> + '_ {
    expect_set(v).map(expect_euid)
}

/// Given an attribute value (`PartialValue`) that we expect to be a set of EUIDs,
/// iterate over those EUIDs, panicking if the attribute value isn't actually a set of EUIDs (or is unknown)
pub fn pv_expect_set_euids(v: &PartialValue) -> impl Iterator<Item = Arc<EntityUID>> + '_ {
    expect_set_euids(pv_expect_known(v))
}
