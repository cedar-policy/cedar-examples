use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cedar_policy::{Policy, PolicyId};
use itertools::Either;
use serde_json::json;
use tracing::info;

use crate::{
    context,
    entitystore::EntityStore,
    objects::Timebox,
    policy_store::PolicyStore,
    util::{EntityUid, ListUid, TimeBoxUid, UserOrTeamUid, TYPE_TIMEBOX, TYPE_USER},
};

#[tracing::instrument]
pub fn update_timebox(
    entities: &mut EntityStore,
    policies: &mut PolicyStore,
    target: UserOrTeamUid,
    list: ListUid,
    d: Option<Duration>,
) -> context::Result<()> {
    let timebox = get_timebox(entities, policies, &target, &list)?;
    if let Some(dur) = d {
        let now = SystemTime::now();
        let then = now.checked_add(dur).unwrap();
        let now_timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let then_timestamp = then.duration_since(UNIX_EPOCH).unwrap().as_secs();
        timebox.set_range(now_timestamp, then_timestamp);
    } else {
        timebox.clear_range();
    }
    Ok(())
}

fn get_timebox<'a>(
    entities: &'a mut EntityStore,
    policies: &mut PolicyStore,
    target: &UserOrTeamUid,
    list: &ListUid,
) -> context::Result<&'a mut Timebox> {
    if entities.get_timebox_mut(target, list).is_none() {
        let id = policies.fresh_policy_id();
        let policy = create_timebox(entities, id, target.clone(), list.clone())?;
        policies.add_dynamic_policy(policy)?;
    }
    Ok(entities.get_timebox_mut(target, list).unwrap())
}

#[tracing::instrument]
fn create_timebox(
    entities: &mut EntityStore,
    id: PolicyId,
    target: UserOrTeamUid,
    list: ListUid,
) -> context::Result<Policy> {
    info!("Creating a new timebox entity & policy");
    let uid: TimeBoxUid = entities.fresh_euid(TYPE_TIMEBOX.clone()).unwrap();
    let timebox = match target.or() {
        Either::Left(user) => Timebox::with_user(uid, user, list),
        Either::Right(team) => Timebox::with_team(uid, team, list),
    };
    let p = create_timebox_policy(id, &timebox)?;
    entities.insert_timebox(timebox);
    Ok(p)
}

#[tracing::instrument]
fn create_timebox_policy(id: PolicyId, t: &Timebox) -> context::Result<Policy> {
    let p = instantiate_timebox_policy(id, t.uid().as_ref(), t.target(), t.list().as_ref());
    info!("Timebox policy: {p}");
    Ok(p)
}

fn instantiate_timebox_policy(
    id: PolicyId,
    timebox: &EntityUid,
    target: &EntityUid,
    list: &EntityUid,
) -> Policy {
    let op = if target.type_name() == &*TYPE_USER {
        "=="
    } else {
        "in"
    };

    let principal_constraint = json!({
        "op" : op,
        "entity" : { "type" : target.type_name().to_string(), "id" : target.id().to_string() }
    });
    let action_constraint = json!({
        "op" : "in",
        "entities" : [
            { "type" : "Action", "id" : "GetList" },
        ]
    });
    let resource_constraint = json!({
        "op" : "==",
        "entity"  : { "type" : list.type_name().to_string(), "id" : list.id().to_string() }
    });

    let timebox = json!({
        "Value" : {
            "__entity" : {
                "type" : timebox.type_name().to_string(),
                "id" : timebox.id().to_string(),
            }
        }
    });

    let now = json!({"." : {
        "left" : { "Var" : "context"},
        "attr" : "now"
    }});

    let timebox_range = json!({
        "." : {
            "left" : timebox,
            "attr" : "range"
        }
    });

    let start_time = json!({
        "." : {
            "left" : timebox_range,
            "attr" : "start"
        }
    });

    let end_time = json!({
        "." : {
            "left" : timebox_range,
            "attr" : "end"
        }
    });

    let after_start = json!({
        "<" : {
            "left" : start_time,
            "right" : now,
        }
    });

    let before_end = json!({
        "<" : {
            "left" : now,
            "right" : end_time
        }
    });

    let principal = json!({"Var" : "principal"});
    let resource = json!({"Var" : "resource"});
    let readers = json!({
        "." : {
            "left" : resource,
            "attr" : "timeboxedReaders"
        }
    });

    let in_readers = json!({
        "in" : {
            "left" : principal,
            "right" : readers
        }
    });

    let when = json!({
        "kind" : "when",
        "body" : {
            "&&" : {
                "left": in_readers,
                "right" : {
                "&&" : {
                    "left" : after_start,
                    "right" : before_end
                }
            }
        }
        }
    });

    let est = json!({
        "effect" : "permit",
        "principal" : principal_constraint,
        "action" : action_constraint,
        "resource" : resource_constraint,
        "conditions" : [when]
    });

    cedar_policy::Policy::from_json(Some(id), est).unwrap()
}
