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

use itertools::Itertools;
use lazy_static::lazy_static;
use std::path::PathBuf;
use tracing::{info, trace};

use cedar_policy::{
    Diagnostics, EntityTypeName, ParseErrors, PolicySet, Schema, SchemaError, ValidationMode,
    Validator,
};
use thiserror::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot,
};

use crate::{
    api::{
        AddShare, CreateList, CreateTask, DeleteList, DeleteShare, DeleteTask, Empty, GetList,
        GetLists, UpdateList, UpdateTask,
    },
    entitystore::{EntityDecodeError, EntityStore},
    objects::List,
    policy_store,
    util::{EntityUid, ListUid, Lists, TYPE_LIST},
    witnesses::{self, actions, Action, AuthWitness},
};

// There's almost certainly a nicer way to do this than having separate `sender` fields

#[derive(Debug)]
pub enum AppResponse {
    GetList(Box<List>),
    Euid(EntityUid),
    Lists(Lists),
    TaskId(i64),
    Unit(()),
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

pub type Result<T> = std::result::Result<T, Error>;

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
    pub static ref ACTION_EDIT_SHARE: EntityUid = r#"Action::"EditShare""#.parse().unwrap();
    pub static ref ACTION_UPDATE_TASK: EntityUid = r#"Action::"UpdateTask""#.parse().unwrap();
    pub static ref ACTION_CREATE_TASK: EntityUid = r#"Action::"CreateTask""#.parse().unwrap();
    pub static ref ACTION_DELETE_TASK: EntityUid = r#"Action::"DeleteTask""#.parse().unwrap();
    pub static ref ACTION_GET_LISTS: EntityUid = r#"Action::"GetLists""#.parse().unwrap();
    pub static ref ACTION_GET_LIST: EntityUid = r#"Action::"GetList""#.parse().unwrap();
    pub static ref ACTION_CREATE_LIST: EntityUid = r#"Action::"CreateList""#.parse().unwrap();
    pub static ref ACTION_UPDATE_LIST: EntityUid = r#"Action::"UpdateList""#.parse().unwrap();
    pub static ref ACTION_DELETE_LIST: EntityUid = r#"Action::"DeleteList""#.parse().unwrap();
}

pub struct AppContext {
    entities: EntityStore,
    policies: PolicySet,
    schema: Schema,
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
        let validator = Validator::new(schema.clone());
        let output = validator.validate(&policies, ValidationMode::default());
        if output.validation_passed() {
            info!("Validation passed!");
            let (send, recv) = tokio::sync::mpsc::channel(100);
            let tx = send.clone();
            tokio::spawn(async move {
                info!("Serving application server!");
                policy_store::spawn_watcher(policies_path, schema_path, tx).await;
                let c = Self {
                    entities,
                    policies,
                    schema,
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
        let proof = self.is_authorized::<actions::EditShare>(&r.uid, &r.list)?;
        let list = self.entities.get_list(&r.list, &proof)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.share_with, proof)?;
        target_entity.insert_parent(team_uid);
        Ok(AppResponse::Unit(()))
    }

    fn delete_share(&mut self, r: DeleteShare) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::EditShare>(&r.uid, &r.list)?;
        let list = self.entities.get_list(&r.list, &proof)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.unshare_with, proof)?;
        target_entity.delete_parent(&team_uid);
        Ok(AppResponse::Unit(()))
    }

    fn update_task(&mut self, r: UpdateTask) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::UpdateTask>(&r.uid, &r.list)?;
        let list = self.entities.get_list_mut(&r.list, &proof)?;
        let task = list
            .get_task_mut(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list.into(), r.task))?;
        if let Some(state) = r.state {
            task.set_state(state);
        }
        if let Some(name) = r.name {
            task.set_name(name);
        }
        Ok(AppResponse::Unit(()))
    }

    fn create_task(&mut self, r: CreateTask) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::CreateTask>(&r.uid, &r.list)?;
        let list = self.entities.get_list_mut(&r.list, &proof)?;
        let task_id = list.create_task(r.name);
        Ok(AppResponse::TaskId(task_id))
    }

    fn delete_task(&mut self, r: DeleteTask) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::DeleteTask>(&r.uid, &r.list)?;
        let list = self.entities.get_list_mut(&r.list, &proof)?;
        list.delete_task(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list.into(), r.task))?;
        Ok(AppResponse::Unit(()))
    }

    fn get_lists(&self, r: GetLists) -> Result<AppResponse> {
        let t: EntityTypeName = "List".parse().unwrap();
        let proof = self.is_authorized::<actions::GetLists>(&r.uid, &*APPLICATION_TINY_TODO)?;

        Ok(AppResponse::Lists(
            self.entities
                .euids(proof)
                .filter(|euid| euid.type_name() == &t)
                .filter(|euid| self.is_authorized::<actions::GetList>(&r.uid, euid).is_ok())
                .cloned()
                .collect::<Vec<EntityUid>>()
                .into(),
        ))
    }

    fn create_list(&mut self, r: CreateList) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::CreateList>(&r.uid, &*APPLICATION_TINY_TODO)?;

        let euid = self
            .entities
            .fresh_euid::<ListUid>(TYPE_LIST.clone())
            .unwrap();
        let l = List::new(&mut self.entities, euid.clone(), r.uid, r.name, &proof);
        self.entities.insert_list(l, proof);

        Ok(AppResponse::euid(euid))
    }

    fn get_list(&self, r: GetList) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::GetList>(&r.uid, &r.list)?;
        let list = self.entities.get_list(&r.list, &proof)?.clone();
        Ok(AppResponse::GetList(Box::new(list)))
    }

    fn update_list(&mut self, r: UpdateList) -> Result<AppResponse> {
        let proof = self.is_authorized::<actions::UpdateList>(&r.uid, &r.list)?;
        let list = self.entities.get_list_mut(&r.list, &proof)?;
        list.update_name(r.name);
        Ok(AppResponse::Unit(()))
    }

    fn delete_list(&mut self, r: DeleteList) -> Result<AppResponse> {
        let witness = self.is_authorized(&r.uid, &r.list)?;
        self.entities.delete_entity(&r.list, witness)?;
        Ok(AppResponse::Unit(()))
    }

    #[tracing::instrument(skip_all)]
    pub fn is_authorized<A: Action>(
        &self,
        principal: impl AsRef<EntityUid>,
        resource: impl AsRef<EntityUid>,
    ) -> Result<AuthWitness<A>> {
        let es = self.entities.as_entities(&self.schema);
        // info!(
        //     "is_authorized request: principal: {}, action: {}, resource: {}",
        //     principal.as_ref(),
        //     action.as_ref(),
        //     resource.as_ref()
        // );
        witnesses::is_authorized(principal, resource, es, &self.policies)
    }
}
