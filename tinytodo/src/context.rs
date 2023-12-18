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
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use tracing::{error, info, trace};

use cedar_policy::{
    Authorizer, Context, Decision, Diagnostics, EntityTypeName, ParseErrors, PolicyId, PolicySet,
    PolicySetError, Request, Schema, SchemaError, SlotId, ValidationMode, Validator,
};
use thiserror::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot,
};

use crate::{
    api::{
        AddShare, CreateList, CreateTask, DeleteList, DeleteShare, DeleteTask, Empty, GetList,
        GetLists, ShareRole, UpdateList, UpdateTask,
    },
    entitystore::{EntityDecodeError, EntityStore},
    objects::List,
    policy_store,
    util::{EntityUid, ListUid, Lists, TYPE_LIST},
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

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Error Parsing Schema: {0}")]
    Schema(#[from] SchemaError),
    #[error("Error Parsing PolicySet: {0}")]
    Policy(#[from] ParseErrors),
    #[error("Error Processing PolicySet: {0}")]
    PolicySet(#[from] PolicySetError),
    #[error("Validation Failed: {0}")]
    Validation(String),
    #[error("Error Deserializing Json: {0}")]
    Json(#[from] serde_json::Error),
}

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
    #[error("Error updating PolicySet: {0}")]
    PolicySet(#[from] PolicySetError),
    #[error("Error constructing authorization request: {0}")]
    Request(String),
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
    static ref ACTION_GET_LIST: EntityUid = r#"Action::"GetList""#.parse().unwrap();
    static ref ACTION_CREATE_LIST: EntityUid = r#"Action::"CreateList""#.parse().unwrap();
    static ref ACTION_UPDATE_LIST: EntityUid = r#"Action::"UpdateList""#.parse().unwrap();
    static ref ACTION_DELETE_LIST: EntityUid = r#"Action::"DeleteList""#.parse().unwrap();
}

pub struct AppContext {
    entities: EntityStore,
    authorizer: Authorizer,
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
enum ReadError {
    #[error("{0}")]
    Parse(#[from] ParseErrors),
    #[error("{0}")]
    Semantics(#[from] PolicySetError),
}

impl From<ReadError> for ContextError {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::Parse(e) => ContextError::Policy(e),
            ReadError::Semantics(e) => ContextError::PolicySet(e),
        }
    }
}

impl From<ReadError> for Error {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::Parse(e) => Error::Policy(e),
            ReadError::Semantics(e) => Error::PolicySet(e),
        }
    }
}
/// Renames policies and templates based on (@id("new_id") annotation.
/// If no such annotation exists, it keeps the current id.
///
/// This will rename template-linked policies to the id of their template, which may
/// cause id conflicts, so only call this function before linking
/// templates into the policy set.
fn rename_from_id_annotation(ps: PolicySet) -> std::result::Result<PolicySet, ReadError> {
    let mut new_ps = PolicySet::new();
    let t_iter = ps.templates().map(|t| match t.annotation("id") {
        None => Ok(t.clone()),
        Some(anno) => {
            //info!("Found template with ID {}!",anno);
            anno.parse().map(|a| t.new_id(a))
        }
    });
    for t in t_iter {
        let template = t?;
        new_ps.add_template(template)?;
    }
    let p_iter = ps.policies().map(|p| match p.annotation("id") {
        None => Ok(p.clone()),
        Some(anno) => {
            //info!("Found policy with ID {}!",anno);
            anno.parse().map(|a| p.new_id(a))
        }
    });
    for p in p_iter {
        let policy = p?;
        new_ps.add(policy)?;
    }
    Ok(new_ps)
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
        let policies0 = policy_src.parse()?;
        let policies = rename_from_id_annotation(policies0)?;
        let validator = Validator::new(schema.clone());
        let output = validator.validate(&policies, ValidationMode::default());
        if output.validation_passed() {
            info!("Validation passed!");
            let authorizer = Authorizer::new();
            let (send, recv) = tokio::sync::mpsc::channel(100);
            let tx = send.clone();
            tokio::spawn(async move {
                info!("Serving application server!");
                // FIXME: probably should pass in [schema] not [schema_path]
                policy_store::spawn_watcher(policies_path, schema_path, tx).await;
                let c = Self {
                    entities,
                    authorizer,
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
        let mut new_policies = rename_from_id_annotation(policy_set)?;
        let mut err = false;
        let mut updated = false;
        // for each existing template-linked policy,
        //   link against the new version of the template in the new policy set if present
        for p in self.policies.policies() {
            match p.template_id() {
                None => (), // not a template-linked policy
                Some(tid) => {
                    // template-linked policy
                    match new_policies.template(tid) {
                        None => {
                            // template not in new policy set
                            let tidx = tid.clone();
                            let pidx = p.id().clone();
                            err = true;
                            error!("Error when reloading policies: Could not find policy template {tidx} to link {pidx}")
                        }
                        Some(_) => {
                            // found template in new policy set
                            match p.template_links() {
                                None => error!("Error when reloading policies: Template with no matching links"),
                                Some(vals) => {
                                    // link against new template, using the same policy ID as the old one
                                    updated = true;
                                    new_policies.link(tid.clone(), p.id().clone(), vals)?
                                }
                            }
                        }
                    }
                }
            }
        }
        // no error during relinking
        if !err {
            // check that re-linked policies validate properly
            if updated {
                let validator = Validator::new(self.schema.clone());
                let output = validator.validate(&new_policies, ValidationMode::default());
                if !output.validation_passed() {
                    for e in output.validation_errors() {
                        error!("Error validating linked policies: {e}")
                    }
                } else {
                    self.policies = new_policies;
                    info!("Reloaded policy set")
                }
            }
        };
        Ok(AppResponse::Unit(()))
    }

    fn add_share(&mut self, r: AddShare) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_EDIT_SHARE, &r.list)?;
        // Confirm that the identified list and sharer are known
        let _list = self.entities.get_list(&r.list)?;
        let _target_entity = self.entities.get_user_or_team_mut(&r.share_with)?;
        // Link a template to register the new permission
        let (tid, pid_prefix) = match r.role {
            ShareRole::Reader => (PolicyId::from_str("reader-template")?, "reader"),
            ShareRole::Editor => (PolicyId::from_str("editor-template")?, "editor"),
        };
        // Construct template linking values
        let target_euid: &cedar_policy::EntityUid = r.share_with.as_ref();
        let list_euid: &cedar_policy::EntityUid = r.list.as_ref();
        let env: HashMap<SlotId, cedar_policy::EntityUid> = [
            (SlotId::principal(), target_euid.clone()),
            (SlotId::resource(), list_euid.clone()),
        ]
        .into_iter()
        .collect();
        // Construct policy ID; assumes no policy in the set has it already
        let target_eid = target_euid.id();
        let list_eid = list_euid.id();
        let pid = PolicyId::from_str(&format!("{pid_prefix}[{target_eid}][{list_eid}]"))?;
        // Link it!
        self.policies.link(tid, pid.clone(), env)?;
        info!("Created policy {pid}");
        Ok(AppResponse::Unit(()))
    }

    fn delete_share(&mut self, r: DeleteShare) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_EDIT_SHARE, &r.list)?;
        // Confirm that the identified list and un-sharer are known
        let _list = self.entities.get_list(&r.list)?;
        let _target_entity = self.entities.get_user_or_team_mut(&r.unshare_with)?;
        // Unlink the policy that provided the permission
        let pid_prefix = match r.role {
            ShareRole::Reader => "reader",
            ShareRole::Editor => "editor",
        };
        let target_eid = r.unshare_with.as_ref().id();
        let list_eid = r.list.as_ref().id();
        let pid = PolicyId::from_str(&format!("{pid_prefix}[{target_eid}][{list_eid}]"))?;
        self.policies.unlink(pid)?;
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
        Ok(AppResponse::Unit(()))
    }

    fn create_task(&mut self, r: CreateTask) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_CREATE_TASK, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        let task_id = list.create_task(r.name);
        Ok(AppResponse::TaskId(task_id))
    }

    fn delete_task(&mut self, r: DeleteTask) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_DELETE_TASK, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        list.delete_task(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list.into(), r.task))?;
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
        let l = List::new(euid.clone(), r.uid, r.name);
        self.entities.insert_list(l);

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
        Ok(AppResponse::Unit(()))
    }

    fn delete_list(&mut self, r: DeleteList) -> Result<AppResponse> {
        self.is_authorized(&r.uid, &*ACTION_DELETE_LIST, &r.list)?;
        self.entities.delete_entity(&r.list)?;
        Ok(AppResponse::Unit(()))
    }

    #[tracing::instrument(skip_all)]
    pub fn is_authorized(
        &self,
        principal: impl AsRef<EntityUid>,
        action: impl AsRef<EntityUid>,
        resource: impl AsRef<EntityUid>,
    ) -> Result<()> {
        let es = self.entities.as_entities(&self.schema);
        let q = Request::new(
            Some(principal.as_ref().clone().into()),
            Some(action.as_ref().clone().into()),
            Some(resource.as_ref().clone().into()),
            Context::empty(),
            Some(&self.schema),
        )
        .map_err(|e| Error::Request(e.to_string()))?;
        info!(
            "is_authorized request: principal: {}, action: {}, resource: {}",
            principal.as_ref(),
            action.as_ref(),
            resource.as_ref()
        );
        let response = self.authorizer.is_authorized(&q, &self.policies, &es);
        info!("Auth response: {:?}", response);
        match response.decision() {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::AuthDenied(response.diagnostics().clone())),
        }
    }
}
