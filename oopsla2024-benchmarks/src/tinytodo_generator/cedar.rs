use arbitrary::{Arbitrary, Result, Unstructured};
use cedar_policy_core::ast::{Context, RequestSchemaAllPass};
use cedar_policy_core::entities::{Entities as CoreEntities, NoEntitiesSchema, TCComputation};
use cedar_policy_core::extensions::Extensions;
use cedar_policy_generators::{hierarchy::Hierarchy, schema::Schema as GeneratorSchema};
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ExampleApp;

use super::{
    common::{make_uid, Uid},
    constants::{NUM_REQUESTS, TINYTODO_SCHEMA_PATH},
    entities::Entities,
};

#[derive(Debug, Clone, Serialize)]
pub struct CedarOutput {
    pub entities: Entities,
    pub requests: Vec<Request>,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum RequestKind {
    Success,
    Random,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum ActionKind {
    Read,
    Write,
    Public,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum Coin {
    Head,
    Tail,
}

lazy_static! {
    static ref ACTIONS: Vec<Uid> = vec![
        make_uid("Action", "CreateList"),
        make_uid("Action", "GetList"),
        make_uid("Action", "UpdateList"),
        make_uid("Action", "DeleteList"),
        make_uid("Action", "GetLists"),
        make_uid("Action", "CreateTask"),
        make_uid("Action", "UpdateTask"),
        make_uid("Action", "DeleteTask"),
        make_uid("Action", "EditShares"),
    ];
    static ref PUBLIC_ACTIONS: Vec<Uid> = vec![
        make_uid("Action", "CreateList"),
        make_uid("Action", "GetLists"),
    ];
    static ref READ_ACTIONS: Vec<Uid> = vec![make_uid("Action", "GetList"),];
    static ref WRITE_ACTIONS: Vec<Uid> = vec![
        make_uid("Action", "UpdateList"),
        make_uid("Action", "DeleteList"),
        make_uid("Action", "CreateTask"),
        make_uid("Action", "UpdateTask"),
        make_uid("Action", "DeleteTask"),
        make_uid("Action", "EditShares"),
    ];
}

impl<'a> Arbitrary<'a> for CedarOutput {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let entities: Entities = u.arbitrary()?;
        let hierarchy = CoreEntities::from_entities(
            entities.to_cedar_entities(),
            None::<&NoEntitiesSchema>,
            TCComputation::ComputeNow,
            Extensions::all_available(),
        )
        .unwrap()
        .into();
        let schema = ExampleApp::load_schema(TINYTODO_SCHEMA_PATH, u);
        let num_requests = NUM_REQUESTS.with(|num_requests| *num_requests.borrow());
        let mut requests = Vec::with_capacity(num_requests);
        for _ in 0..num_requests {
            requests.push(generate_request(u, &schema, &hierarchy)?);
        }
        Ok(Self { entities, requests })
    }
}

pub fn generate_request(
    u: &mut Unstructured<'_>,
    schema: &GeneratorSchema,
    hierarchy: &Hierarchy,
) -> Result<Request> {
    let r = loop {
        let request = schema.arbitrary_request(hierarchy, u)?;
        if hierarchy
            .uids()
            .iter()
            .any(|h_uid| &request.principal == h_uid)
            && hierarchy
                .uids()
                .iter()
                .any(|h_uid| &request.resource == h_uid)
        {
            break request;
        }
    };
    let r = Request {
        principal: r.principal.clone().into(),
        action: r.action.clone().into(),
        resource: r.resource.clone().into(),
        context: HashMap::default(),
    };
    Ok(r)
}

#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub principal: Uid,
    pub action: Uid,
    pub resource: Uid,
    // This will always be the empty hashmap
    context: HashMap<(), ()>,
}

impl Request {
    pub fn to_cedar_request(&self) -> cedar_policy_core::ast::Request {
        cedar_policy_core::ast::Request::new(
            self.principal.to_euid(),
            self.action.to_euid(),
            self.resource.to_euid(),
            Context::empty(),
            None::<&RequestSchemaAllPass>,
            Extensions::all_available(),
        )
        .expect("failed to construct request")
    }
}

impl From<cedar_policy_core::ast::Request> for Request {
    fn from(r: cedar_policy_core::ast::Request) -> Self {
        let expect_concrete = |uidentry: &cedar_policy_core::ast::EntityUIDEntry| -> Arc<cedar_policy_core::ast::EntityUID> { match uidentry {
            cedar_policy_core::ast::EntityUIDEntry::Known(euid) => Arc::clone(euid),
            cedar_policy_core::ast::EntityUIDEntry::Unknown => panic!("expected concrete entry"),
        }};
        let principal = expect_concrete(r.principal());
        let action = expect_concrete(r.action());
        let resource = expect_concrete(r.resource());
        Request {
            principal: Uid::from(&*principal),
            action: Uid::from(&*action),
            resource: Uid::from(&*resource),
            context: HashMap::default(),
        }
    }
}
