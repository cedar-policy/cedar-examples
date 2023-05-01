use itertools::Itertools;
use std::path::Path;

use cedar_policy::{
    Authorizer, Context, Decision, Diagnostics, EntityTypeName, ParseErrors, PolicySet, Request,
    Schema, SchemaError, ValidationMode, Validator,
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
    util::{EntityUid, Lists},
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
}

pub struct AppContext {
    entities: EntityStore,
    authorizer: Authorizer,
    policies: PolicySet,
    recv: Receiver<AppQuery>,
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
    pub fn spawn(
        entities_path: impl AsRef<Path>,
        schema_path: impl AsRef<Path>,
        policies_path: impl AsRef<Path>,
    ) -> std::result::Result<Sender<AppQuery>, ContextError> {
        let schema_file = std::fs::File::open(schema_path)?;
        let schema = Schema::from_file(schema_file)?;

        let entities_file = std::fs::File::open(entities_path)?;
        let entities = serde_json::from_reader(entities_file)?;

        let policy_src = std::fs::read_to_string(policies_path)?;
        let policies = policy_src.parse()?;
        let validator = Validator::new(schema);
        let output = validator.validate(&policies, ValidationMode::default());
        if output.validation_passed() {
            let authorizer = Authorizer::new();
            let (send, recv) = tokio::sync::mpsc::channel(100);

            tokio::spawn(async move {
                let c = Self {
                    entities,
                    authorizer,
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
                };
                msg.sender.send(r).unwrap();
            }
        }
    }

    fn add_share(&mut self, r: AddShare) -> Result<AppResponse> {
        let action = r#"Action::"EditShare""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list(&r.list)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.share_with)?;
        target_entity.add_parent(team_uid);
        Ok(AppResponse::Unit(()))
    }

    fn delete_share(&mut self, r: DeleteShare) -> Result<AppResponse> {
        let action = r#"Action::"EditShare""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list(&r.list)?;
        let team_uid = list.get_team(r.role).clone();
        let target_entity = self.entities.get_user_or_team_mut(&r.unshare_with)?;
        target_entity.remove_parent(&team_uid);
        Ok(AppResponse::Unit(()))
    }

    fn update_task(&mut self, r: UpdateTask) -> Result<AppResponse> {
        let action: EntityUid = r#"Action::"UpdateTask""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        let task = list
            .get_task_mut(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list, r.task))?;
        if let Some(state) = r.state {
            task.set_state(state);
        }
        if let Some(name) = r.description {
            task.set_name(name);
        }
        Ok(AppResponse::Unit(()))
    }

    fn create_task(&mut self, r: CreateTask) -> Result<AppResponse> {
        let action: EntityUid = r#"Action::"CreateTask""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        let task_id = list.create_task(r.description);
        Ok(AppResponse::TaskId(task_id))
    }

    fn delete_task(&mut self, r: DeleteTask) -> Result<AppResponse> {
        let action = r#"Action::"DeleteTask""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        list.delete_task(r.task)
            .ok_or_else(|| Error::InvalidTaskId(r.list, r.task))?;
        Ok(AppResponse::Unit(()))
    }

    fn get_lists(&self, r: GetLists) -> Result<AppResponse> {
        let t: EntityTypeName = "List".parse().unwrap();
        let action = r#"Action::"GetLists""#.parse().unwrap();
        let application = r#"Application::"TinyTodo""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &application)?;

        let action: EntityUid = r#"Action::"GetList""#.parse().unwrap();
        Ok(AppResponse::Lists(
            self.entities
                .euids()
                .filter(|euid| euid.type_name() == &t)
                .filter(|euid| self.is_authorized(&r.uid, &action, euid).is_ok())
                .cloned()
                .collect::<Vec<EntityUid>>()
                .into(),
        ))
    }

    fn create_list(&mut self, r: CreateList) -> Result<AppResponse> {
        let action = r#"Action::"CreateList""#.parse().unwrap();
        let application = r#"Application::"TinyTodo""#.parse().unwrap();

        self.is_authorized(&r.uid, &action, &application)?;

        let type_name = "List".parse().unwrap();
        let euid = self.entities.fresh_euid(type_name);
        let l = List::new(&mut self.entities, euid.clone(), r.uid, r.name);
        self.entities.insert_list(l);

        Ok(AppResponse::Euid(euid))
    }

    fn get_list(&self, r: GetList) -> Result<AppResponse> {
        let action = r#"Action::"GetList""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list_id)?;
        let list = self.entities.get_list(&r.list_id)?.clone();
        Ok(AppResponse::GetList(Box::new(list)))
    }

    fn update_list(&mut self, r: UpdateList) -> Result<AppResponse> {
        let action = r#"Action::"UpdateList""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        let list = self.entities.get_list_mut(&r.list)?;
        list.update_name(r.name);
        Ok(AppResponse::Unit(()))
    }

    fn delete_list(&mut self, r: DeleteList) -> Result<AppResponse> {
        let action = r#"Action::"DeleteList""#.parse().unwrap();
        self.is_authorized(&r.uid, &action, &r.list)?;
        self.entities.delete_entity(&r.list)?;
        Ok(AppResponse::Unit(()))
    }

    pub fn is_authorized(
        &self,
        principal: &EntityUid,
        action: &EntityUid,
        resource: &EntityUid,
    ) -> Result<()> {
        let es = self.entities.as_entities();
        let q = Request::new(
            Some(principal.clone().into()),
            Some(action.clone().into()),
            Some(resource.clone().into()),
            Context::empty(),
        );
        let ans = self.authorizer.is_authorized(&q, &self.policies, &es);
        match ans.decision() {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::AuthDenied(ans.diagnostics().clone())),
        }
    }
}
