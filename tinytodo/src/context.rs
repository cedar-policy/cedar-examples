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

use cedar_policy::{
    Diagnostics, EntityTypeName, ParseErrors, PolicySet, Schema, SchemaError, ValidationMode,
    Validator,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::io::Cursor;
use std::{collections::HashMap, io::Read, path::PathBuf};
use thiserror::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot,
};
use tracing::{error, info, log::warn, trace};

use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AddShare, CreateList, CreateTask, DeleteList, DeleteShare, DeleteTask, Empty, GetList,
        GetLists, UpdateList, UpdateTask,
    },
    entitystore::{EntityDecodeError, EntityStore},
    objects::List,
    policy_store,
    util::{EntityUid, ListUid, Lists, TYPE_LIST},
};

#[derive(Debug, Serialize, Deserialize)]
struct AuthDecision {
    decision: String,
    diagnostics: Diagnostics,
}

// There's almost certainly a nicer way to do this than having separate `sender` fields

#[derive(Debug)]
pub enum AppResponse {
    GetList(Box<List>),
    Euid(EntityUid),
    Lists(Lists),
    PolicyList(PolicySet),
    EntityList(EntityList),
    TaskId(i64),
    Unit(()),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityList(Vec<PolicyEntity>);

#[derive(Debug, Serialize, Deserialize)]
struct PolicyEntity {
    uid: PolicyEntityUid,
    attrs: HashMap<String, serde_json::Value>,
    parents: Vec<PolicyEntityUid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolicyAttrs {
    // optional fields readers and editors
    #[serde(rename = "readers")]
    readers: Option<PolicyAttr>,
    #[serde(rename = "editors")]
    editors: Option<PolicyAttr>,
    #[serde(rename = "owner")]
    owner: Option<PolicyAttr>,
    #[serde(rename = "name")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolicyAttr {
    #[serde(rename = "__entity")]
    entity: PolicyEntityUid,
    name: String,
    tasks: Vec<PolicyEntityTask>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolicyEntityUid {
    #[serde(rename = "type")]
    entity_type: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolicyEntityTask {
    #[serde(rename = "__entity")]
    entity: PolicyEntity,
}

impl AppResponse {
    pub fn euid(v: impl Into<EntityUid>) -> Self {
        Self::Euid(v.into())
    }
}

impl TryInto<i64> for AppResponse {
    type Error = Error;

    fn try_into(self) -> std::result::Result<i64, Self::Error> {
        match self {
            AppResponse::TaskId(id) => Ok(id),
            _ => Err(Error::Type),
        }
    }
}

impl TryInto<List> for AppResponse {
    type Error = Error;

    fn try_into(self) -> std::result::Result<List, Self::Error> {
        match self {
            AppResponse::GetList(l) => Ok(*l),
            _ => Err(Error::Type),
        }
    }
}

impl TryInto<EntityUid> for AppResponse {
    type Error = Error;
    fn try_into(self) -> std::result::Result<EntityUid, Self::Error> {
        match self {
            AppResponse::Euid(e) => Ok(e),
            _ => Err(Error::Type),
        }
    }
}

impl TryInto<Empty> for AppResponse {
    type Error = Error;

    fn try_into(self) -> std::result::Result<Empty, Self::Error> {
        match self {
            AppResponse::Unit(()) => Ok(Empty::default()),
            _ => Err(Error::Type),
        }
    }
}

impl TryInto<Lists> for AppResponse {
    type Error = Error;
    fn try_into(self) -> std::result::Result<Lists, Self::Error> {
        match self {
            AppResponse::Lists(l) => Ok(l),
            _ => Err(Error::Type),
        }
    }
}

impl TryInto<EntityList> for AppResponse {
    type Error = Error;
    fn try_into(self) -> std::result::Result<EntityList, Self::Error> {
        match self {
            AppResponse::EntityList(l) => Ok(l),
            _ => Err(Error::Type),
        }
    }
}

#[derive(Debug)]
pub enum AppQueryKind {
    // List CRUD
    CreateList(CreateList),
    GetList(GetList),
    UpdateList(UpdateList),
    DeleteList(DeleteList),

    // Task CRUD
    CreateTask(CreateTask),
    UpdateTask(UpdateTask),
    DeleteTask(DeleteTask),

    // Lists
    GetLists(GetLists),

    // Shares
    AddShare(AddShare),
    DeleteShare(DeleteShare),

    // Policy Set Updates
    UpdatePolicySet(PolicySet),

    // Policy Set Queries
    GetPolicies(),

    // Entity Queries
    GetEntities(),
}

#[derive(Debug)]
pub struct AppQuery {
    kind: AppQueryKind,
    sender: oneshot::Sender<Result<AppResponse>>,
}

impl AppQuery {
    pub fn new(kind: AppQueryKind, sender: oneshot::Sender<Result<AppResponse>>) -> Self {
        Self { kind, sender }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No Such Entity: {0}")]
    NoSuchEntity(EntityUid),
    #[error("Entity Decode Error: {0}")]
    EntityDecode(#[from] EntityDecodeError),
    #[error("Authorization Denied")]
    AuthDenied(Diagnostics),
    #[error("The list {0} does not contain a task with id {1}")]
    InvalidTaskId(EntityUid, i64),
    #[error("Internal Error")]
    TokioSend(#[from] tokio::sync::mpsc::error::SendError<AppQuery>),
    #[error("Internal Error")]
    TokioRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Internal Error")]
    Type,
    #[error("Internal Error")]
    IO(#[from] std::io::Error),
    #[error("Error Parsing PolicySet: {0}")]
    Policy(#[from] ParseErrors),
}

impl Error {
    pub fn no_such_entity(euid: impl Into<EntityUid>) -> Self {
        Self::NoSuchEntity(euid.into())
    }
}

lazy_static! {
    pub static ref APPLICATION_TINY_TODO: EntityUid = r#"Application::"TinyTodo""#.parse().unwrap();
    static ref ACTION_EDIT_SHARE: EntityUid = r#"Action::"EditShare""#.parse().unwrap();
    static ref ACTION_UPDATE_TASK: EntityUid = r#"Action::"UpdateTask""#.parse().unwrap();
    static ref ACTION_CREATE_TASK: EntityUid = r#"Action::"CreateTask""#.parse().unwrap();
    static ref ACTION_DELETE_TASK: EntityUid = r#"Action::"DeleteTask""#.parse().unwrap();
    static ref ACTION_GET_LISTS: EntityUid = r#"Action::"GetLists""#.parse().unwrap();
    static ref ACTION_GET_POLICIES: EntityUid = r#"Action::"GetPolicies""#.parse().unwrap();
    static ref ACTION_GET_ENTITIES: EntityUid = r#"Action::"GetEntities""#.parse().unwrap();
    static ref ACTION_GET_LIST: EntityUid = r#"Action::"GetList""#.parse().unwrap();
    static ref ACTION_CREATE_LIST: EntityUid = r#"Action::"CreateList""#.parse().unwrap();
    static ref ACTION_UPDATE_LIST: EntityUid = r#"Action::"UpdateList""#.parse().unwrap();
    static ref ACTION_DELETE_LIST: EntityUid = r#"Action::"DeleteList""#.parse().unwrap();
}

pub struct AppContext {
    entities: EntityStore,
    // authorizer: Authorizer,
    policies: PolicySet,
    recv: Receiver<AppQuery>,
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AppContext>")
    }
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Error Parsing Schema: {0}")]
    Schema(#[from] SchemaError),
    #[error("Error Parsing PolicySet: {0}")]
    Policy(#[from] ParseErrors),
    #[error("Validation Failed: {0}")]
    Validation(String),
    #[error("Error Deserializing Json: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppContext {
    #[tracing::instrument(skip_all)]
    pub fn spawn(
        entities_path: impl Into<PathBuf>,
        schema_path: impl Into<PathBuf>,
        policies_path: impl Into<PathBuf>,
    ) -> std::result::Result<Sender<AppQuery>, ContextError> {
        let schema_path = schema_path.into();
        let policies_path = policies_path.into();
        let schema_file = std::fs::File::open(&schema_path)?;
        let schema = Schema::from_file(schema_file)?;

        let entities_file = std::fs::File::open(entities_path.into())?;
        let entities = serde_json::from_reader(entities_file)?;

        let policy_src = std::fs::read_to_string(&policies_path)?;
        let policies = policy_src.parse()?;
        let validator = Validator::new(schema);
        let output = validator.validate(&policies, ValidationMode::default());
        if output.validation_passed() {
            info!("Validation passed!");
            // let authorizer = Authorizer::new();
            let (send, recv) = tokio::sync::mpsc::channel(100);
            let tx = send.clone();
            tokio::spawn(async move {
                info!("Serving application server!");
                policy_store::spawn_watcher(policies_path, schema_path, tx).await;
                let c = Self {
                    entities,
                    // authorizer,
                    policies,
                    recv,
                };
                c.serve().await
            });

            Ok(send)
        } else {
            let error_string = output
                .validation_errors()
                .map(|err| format!("{err}"))
                .join("\n");
            Err(ContextError::Validation(error_string))
        }
    }

    #[tracing::instrument]
    async fn serve(mut self) -> Result<()> {
        loop {
            if let Some(msg) = self.recv.recv().await {
                let r = match msg.kind {
                    AppQueryKind::GetList(r) => self.get_list(r),
                    AppQueryKind::CreateList(r) => self.create_list(r),
                    AppQueryKind::UpdateList(r) => self.update_list(r),
                    AppQueryKind::DeleteList(r) => self.delete_list(r),
                    AppQueryKind::CreateTask(r) => self.create_task(r),
                    AppQueryKind::UpdateTask(r) => self.update_task(r),
                    AppQueryKind::DeleteTask(r) => self.delete_task(r),
                    AppQueryKind::GetLists(r) => self.get_lists(r),
                    AppQueryKind::AddShare(r) => self.add_share(r),
                    AppQueryKind::DeleteShare(r) => self.delete_share(r),
                    AppQueryKind::UpdatePolicySet(set) => self.update_policy_set(set),
                    // get all policies
                    AppQueryKind::GetPolicies() => self.get_policies(),
                    // get all entities
                    AppQueryKind::GetEntities() => self.get_entities(),
                };
                if let Err(e) = msg.sender.send(r) {
                    trace!("Failed send response: {:?}", e);
                }
            }
        }
    }

    #[tracing::instrument(skip(policy_set))]
    fn update_policy_set(&mut self, policy_set: PolicySet) -> Result<AppResponse> {
        self.policies = policy_set;
        info!("Reloaded policy set");
        Ok(AppResponse::Unit(()))
    }

    fn add_share(&mut self, r: AddShare) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_EDIT_SHARE, &r.list)?;
        let list = self.entities.get_list(&r.list)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.share_with)?;
        target_entity.insert_parent(team_uid);
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    fn get_policies(&self) -> Result<AppResponse> {
        // self.is_authorized(&r.uid, &*ACTION_GET_POLICIES, &r.list)?;
        Ok(AppResponse::PolicyList(self.policies.clone()))
    }

    fn get_entities(&self) -> Result<AppResponse> {
        let es: cedar_policy::Entities = self.entities.as_entities();
        let mut entities_c = Cursor::new(Vec::new());

        let _wtj = es.write_to_json(&mut entities_c);
        let entities_str: String = String::from_utf8(entities_c.into_inner()).unwrap();
        let entities: EntityList = serde_json::from_str(&entities_str).unwrap();

        Ok(AppResponse::EntityList(entities))
    }

    fn save_entities_and_sync(&self) {
        // you can also send the data directly to cedar-agent
        // let es: cedar_policy::Entities = self.entities.as_entities();
        // let mut entities_c = Cursor::new(Vec::new());

        // let _wtj = es.write_to_json(&mut entities_c);
        // let entities_str: String = String::from_utf8(entities_c.into_inner()).unwrap();

        let client = reqwest::blocking::Client::new();
        let res = client
            .post("http://localhost:7002/data/config")
            .json(&serde_json::json!({
                "entries": [{
                    "url": "http://host.docker.internal:8080/api/entities/get",
                    // "data": entities_str,
                    "topics": ["policy_data"],
                    "dst_path": ""
                }]
            }))
            .send();
        match res.is_ok() {
            true => info!("Synced entities to cedar-agent: {:?}", res),
            false => error!("Failed to sync entities to cedar-agent"),
        }
    }

    fn delete_share(&mut self, r: DeleteShare) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_EDIT_SHARE, &r.list)?;
        let list = self.entities.get_list(&r.list)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.unshare_with)?;
        target_entity.delete_parent(&team_uid);
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    fn update_task(&mut self, r: UpdateTask) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_UPDATE_TASK, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        let task = list
            .get_task_mut(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list.into(), r.task))?;
        if let Some(state) = r.state {
            task.set_state(state);
        }
        if let Some(name) = r.name {
            task.set_name(name);
        }
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    fn create_task(&mut self, r: CreateTask) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_CREATE_TASK, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        let task_id = list.create_task(r.name);
        self.save_entities_and_sync();
        Ok(AppResponse::TaskId(task_id))
    }

    fn delete_task(&mut self, r: DeleteTask) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_DELETE_TASK, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        list.delete_task(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list.into(), r.task))?;
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    fn get_lists(&self, r: GetLists) -> Result<AppResponse> {
        let t: EntityTypeName = "List".parse().unwrap();
        self.is_authorized(&r.uid, &*ACTION_GET_LISTS, &*APPLICATION_TINY_TODO)?;

        Ok(AppResponse::Lists(
            self.entities
                .euids()
                .filter(|euid| euid.type_name() == &t)
                .filter(|euid| self.is_authorized(&r.uid, &*ACTION_GET_LIST, euid).is_ok())
                .cloned()
                .collect::<Vec<EntityUid>>()
                .into(),
        ))
    }

    fn create_list(&mut self, r: CreateList) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_CREATE_LIST, &*APPLICATION_TINY_TODO)?;

        let euid = self
            .entities
            .fresh_euid::<ListUid>(TYPE_LIST.clone())
            .unwrap();
        let l = List::new(&mut self.entities, euid.clone(), r.uid, r.name);
        self.entities.insert_list(l);

        self.save_entities_and_sync();
        Ok(AppResponse::euid(euid))
    }

    fn get_list(&self, r: GetList) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_GET_LIST, &r.list)?;
        let list = self.entities.get_list(&r.list)?.clone();
        Ok(AppResponse::GetList(Box::new(list)))
    }

    fn update_list(&mut self, r: UpdateList) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_UPDATE_LIST, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        list.update_name(r.name);
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    fn delete_list(&mut self, r: DeleteList) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_DELETE_LIST, &r.list)?;
        self.entities.delete_entity(&r.list)?;
        self.save_entities_and_sync();
        Ok(AppResponse::Unit(()))
    }

    #[tracing::instrument(skip_all)]
    pub fn is_authorized(
        &self,
        principal: impl AsRef<EntityUid>,
        action: impl AsRef<EntityUid>,
        resource: impl AsRef<EntityUid>,
    ) -> Result<()> {
        // let es = self.entities.as_entities();
        // let q: Request = Request::new(
        //     Some(principal.as_ref().clone().into()),
        //     Some(action.as_ref().clone().into()),
        //     Some(resource.as_ref().clone().into()),
        //     Context::empty(),
        // );
        info!(
            "is_authorized request: principal: {}, action: {}, resource: {}",
            principal.as_ref(),
            action.as_ref(),
            resource.as_ref()
        );

        // let response = self.authorizer.is_authorized(&q, &self.policies, &es);

        // info!("Auth response: {:?}", response);
        // match response.decision() {
        //     Decision::Allow => Ok(()),
        //     Decision::Deny => Err(Error::AuthDenied(response.diagnostics().clone())),
        // }

        // let params = [("principal", principal.as_ref().clone().to_string()),
        //  ("action", action.as_ref().clone().to_string()),
        //  ("resource", resource.as_ref().clone().to_string()),
        //  ("context", "{}".to_string())
        //  ];

        let client = reqwest::blocking::Client::new();
        let res = client
            .post("http://localhost:8180/v1/is_authorized")
            .json(&serde_json::json!({
                "principal": principal.as_ref().clone().to_string(),
                "action": action.as_ref().clone().to_string(),
                "resource": resource.as_ref().clone().to_string(),
                "context": {}
            }))
            .send();

        let mut body = String::new();
        match res {
            Ok(mut res) => {
                res.read_to_string(&mut body).unwrap();
                // convert body to AuthDecision struct
                let auth_decision: AuthDecision = serde_json::from_str(&body).unwrap();
                info!("Auth decision: {:?}", auth_decision);
                match auth_decision.decision.as_str() {
                    "Allow" => return Ok(()),
                    "Deny" => return Err(Error::AuthDenied(auth_decision.diagnostics)),
                    _ => return Err(Error::Type),
                }
            }
            Err(e) => {
                info!("Error: {:?}", e);
                return Err(Error::Type);
            }
        }
    }
}
