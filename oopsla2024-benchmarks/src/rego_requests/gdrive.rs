use std::collections::HashMap;

use cedar_policy_core::{
    ast::{self, Entity, EntityUID},
    entities::Entities,
};
use serde::Serialize;

use crate::entity_graph::EntityGraph;

#[derive(Debug, Clone, Serialize)]
pub struct Input {
    #[serde(rename = "Policy")]
    policy: String,
    #[serde(rename = "Namespace")]
    namespace: &'static str,
    #[serde(rename = "Requests")]
    requests: Vec<Request>,
}

impl Input {
    pub fn new(requests: Vec<Request>) -> Self {
        let policy = std::fs::read_to_string("openfga-examples/rego/gdrive.rego").unwrap();
        let namespace = "gdrive";
        Self {
            policy,
            namespace,
            requests,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Request {
    principal: Principal,
    action: String,
    resource: Resource,
    files: FileGraph,
}

impl Request {
    pub fn new(r: &ast::Request, es: &impl EntityGraph, closed: &Entities) -> Self {
        let principal = Principal::new(r.principal().uid().unwrap(), es, closed);
        let action = r.action().uid().unwrap().to_string();
        let resource = Resource::new(r.resource().uid().unwrap().to_string(), es);
        let files = build_file_graph(es.iter());
        Self {
            principal,
            action,
            resource,
            files,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Principal {
    uid: String,
    #[serde(rename = "documentsAndFoldersWithViewAccess")]
    documents_and_folders_with_view_access: Vec<String>,
    #[serde(rename = "ownedFolders")]
    owned_folders: Vec<String>,
    #[serde(rename = "ownedDocuments")]
    owned_documents: Vec<String>,
}

impl Principal {
    pub fn new(euid: &EntityUID, es: &impl EntityGraph, closed: &Entities) -> Self {
        let e = es.get(&euid.to_string()).unwrap();
        let owned_documents = euids_as_strings(e, "ownedDocuments").collect();
        let owned_folders = euids_as_strings(e, "ownedFolders").collect();
        let documents_and_folders_with_view_access = get_view_access(e, closed);
        Self {
            uid: euid.to_string(),
            owned_documents,
            owned_folders,
            documents_and_folders_with_view_access,
        }
    }
}

fn get_view_access(e: &Entity, closed: &Entities) -> Vec<String> {
    let view_euid =
        crate::utils::pv_expect_euid(e.get("documentsAndFoldersWithViewAccess").unwrap());

    let root_view = closed.entity(&view_euid).unwrap();
    root_view
        .ancestors()
        .filter(is_document_or_folder)
        .map(EntityUID::to_string)
        .collect()
}

fn is_document_or_folder(e: &&EntityUID) -> bool {
    let typ = e.entity_type().to_string();
    typ == "Folder" || typ == "Document"
}

fn euids_as_strings<'a>(e: &'a Entity, attr: &str) -> impl Iterator<Item = String> + 'a {
    crate::utils::pv_expect_set_euids(e.get(attr).unwrap()).map(|euid| euid.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct Resource {
    uid: String,
    #[serde(rename = "isPublic")]
    is_public: bool,
}

impl Resource {
    pub fn new(uid: String, es: &impl EntityGraph) -> Self {
        let e = es.get(&uid).unwrap();
        let is_public = e
            .get("isPublic")
            .map(crate::utils::pv_expect_bool)
            .unwrap_or(false);
        Resource { uid, is_public }
    }
}

fn build_file_graph(es: impl IntoIterator<Item = Entity>) -> FileGraph {
    es.into_iter()
        .filter(|e| is_document_or_folder(&&e.uid()))
        .map(|e| {
            (
                e.uid().to_string(),
                e.ancestors().map(|e| e.to_string()).collect(),
            )
        })
        .collect()
}

type FileGraph = HashMap<String, Vec<String>>;
