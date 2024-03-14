use cedar_policy_core::ast::{EntityUID, PartialValue, Request};
use cedar_policy_generators::collections::HashMap;
use serde::Serialize;
use smol_str::SmolStr;

use crate::entity_graph::EntityGraph;

#[derive(Debug, Serialize)]
pub struct GithubOpaInput {
    #[serde(rename = "Policy")]
    policy: String,
    #[serde(rename = "Namespace")]
    namespace: &'static str,
    #[serde(rename = "Requests")]
    requests: Vec<GithubRequest>,
}

impl GithubOpaInput {
    pub fn new(requests: Vec<GithubRequest>) -> Self {
        let policy = std::fs::read_to_string("openfga-examples/rego/github.rego").unwrap();
        let namespace = "github";
        Self {
            policy,
            namespace,
            requests,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GithubRequest {
    pub principal: String,
    pub action: String,
    pub resource: GithubRepo,
    pub orgs: OrgChart,
}

impl GithubRequest {
    pub fn new(r: &Request, es: &impl EntityGraph) -> Self {
        let resource = GithubRepo::new(r.resource().uid().unwrap(), es);
        let orgs = make_org_chart(es);
        Self {
            principal: r.principal().uid().unwrap().to_string(),
            action: r.action().uid().unwrap().to_string(),
            resource,
            orgs,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GithubRepo {
    #[serde(rename = "Action::\"read\"")]
    readers: String,
    #[serde(rename = "Action::\"write\"")]
    writers: String,
    #[serde(rename = "Action::\"admin\"")]
    admins: String,
    #[serde(rename = "Action::\"triage\"")]
    triagers: String,
    #[serde(rename = "Action::\"maintain\"")]
    maintainers: String,
    owner: GithubOrg,
}

impl GithubRepo {
    pub fn new(x: &EntityUID, es: &impl EntityGraph) -> Self {
        let e = es.get(&x.to_string()).unwrap();
        let attrs: HashMap<SmolStr, &PartialValue> =
            e.attrs().map(|(k, v)| (k.clone(), v)).collect();
        Self {
            readers: euid_to_string(attrs.get("readers").unwrap()),
            writers: euid_to_string(attrs.get("writers").unwrap()),
            admins: euid_to_string(attrs.get("admins").unwrap()),
            triagers: euid_to_string(attrs.get("triagers").unwrap()),
            maintainers: euid_to_string(attrs.get("maintainers").unwrap()),
            owner: GithubOrg::new(euid_to_string(attrs.get("owner").unwrap()), es),
        }
    }
}

fn euid_to_string(pval: &PartialValue) -> String {
    crate::utils::pv_expect_euid(pval).to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct GithubOrg {
    #[serde(rename = "Action::\"read\"")]
    readers: String,
    #[serde(rename = "Action::\"write\"")]
    writers: String,
    #[serde(rename = "Action::\"admin\"")]
    admins: String,
}

impl GithubOrg {
    pub fn new(uid: String, es: &impl EntityGraph) -> Self {
        let e = es.get(&uid).unwrap();
        let attrs: HashMap<SmolStr, &PartialValue> =
            e.attrs().map(|(k, v)| (k.clone(), v)).collect();
        Self {
            readers: euid_to_string(attrs.get("readers").unwrap()),
            writers: euid_to_string(attrs.get("writers").unwrap()),
            admins: euid_to_string(attrs.get("admins").unwrap()),
        }
    }
}

pub fn make_org_chart(es: &impl EntityGraph) -> OrgChart {
    es.iter().map(make_org_char_entry).collect()
}

fn make_org_char_entry(e: cedar_policy_core::ast::Entity) -> (String, Vec<String>) {
    let euid = e.uid().to_string();
    let parents: Vec<_> = e.ancestors().map(EntityUID::to_string).collect();
    (euid, parents)
}

pub type OrgChart = HashMap<String, Vec<String>>;
