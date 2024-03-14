use std::{collections::HashMap, fs::File, io::Read};

use serde::Serialize;

use super::{
    cedar::Request,
    common::{application, Entity, Uid},
    constants::TINYTODO_REGO,
    entities::Entities,
};

pub fn to_opa<'a>(
    entities: &Entities,
    requests: impl IntoIterator<Item = &'a Request>,
) -> OpaOutput {
    let data = generate_group_data(entities);
    let requests = requests
        .into_iter()
        .map(|request| make_output(request, &data, entities))
        .collect();

    OpaOutput {
        policy: load_opa_policy(TINYTODO_REGO),
        namespace: "tinytodo",
        requests,
    }
}

fn generate_group_data(e: &Entities) -> OpaData {
    let groups = e
        .teams()
        .map(entity_to_group_entry)
        .chain(e.users().map(entity_to_group_entry))
        .collect();
    OpaData { groups }
}

fn entity_to_group_entry(e: &Entity) -> (Uid, Vec<Uid>) {
    (e.euid.clone(), e.parents.clone())
}

fn load_opa_policy(src: &str) -> String {
    let mut f = match File::open(src) {
        Ok(f) => f,
        Err(e) => panic!("Couldn't read tinytodo rego file {src}: {e}"),
    };
    let mut buf = String::default();
    f.read_to_string(&mut buf).unwrap();
    buf
}

fn make_output(r: &Request, data: &OpaData, es: &Entities) -> OpaField {
    OpaField {
        request: make_request(r, es),
        data: data.clone(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpaOutput {
    #[serde(rename = "Policy")]
    pub policy: String,
    #[serde(rename = "Namespace")]
    pub namespace: &'static str,
    #[serde(rename = "Requests")]
    pub requests: Vec<OpaField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpaField {
    #[serde(rename = "Request")]
    request: OpaRequest,
    #[serde(rename = "Data")]
    data: OpaData,
}

fn make_request(r: &Request, es: &Entities) -> OpaRequest {
    let principal = r.principal.clone();
    let action = r.action.clone();
    let resource = make_resource(&r.resource, es);
    OpaRequest {
        principal,
        action,
        resource,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpaRequest {
    #[serde(rename = "Principal")]
    principal: Uid,
    #[serde(rename = "Action")]
    action: Uid,
    #[serde(rename = "Resource")]
    resource: OpaResource,
}

fn make_resource(uid: &Uid, es: &Entities) -> OpaResource {
    if uid == &application() {
        OpaResource {
            owner: uid.clone(),
            readers: vec![],
            writers: vec![],
        }
    } else {
        let list = es.list(uid).unwrap();
        OpaResource {
            owner: list.owner.clone(),
            readers: vec![list.readers.clone()],
            writers: vec![list.editors.clone()],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpaResource {
    #[serde(rename = "Owner")]
    owner: Uid,
    #[serde(rename = "Readers")]
    readers: Vec<Uid>,
    #[serde(rename = "Writers")]
    writers: Vec<Uid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpaData {
    #[serde(rename = "Groups")]
    groups: HashMap<Uid, Vec<Uid>>,
}
