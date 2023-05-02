use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::{mpsc, oneshot};
use warp::Filter;

use crate::{
    context::{AppQuery, AppQueryKind, AppResponse, Error},
    objects::{List, TaskState},
    util::{EntityUid, Lists},
};

type AppChannel = mpsc::Sender<AppQuery>;

#[derive(Debug, Clone, Deserialize)]
pub struct GetList {
    pub uid: EntityUid,
    pub list_id: EntityUid,
}

impl From<GetList> for AppQueryKind {
    fn from(v: GetList) -> AppQueryKind {
        AppQueryKind::GetList(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateList {
    pub uid: EntityUid,
    pub name: String,
}

impl From<CreateList> for AppQueryKind {
    fn from(v: CreateList) -> AppQueryKind {
        AppQueryKind::CreateList(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateList {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub name: String,
}

impl From<UpdateList> for AppQueryKind {
    fn from(v: UpdateList) -> AppQueryKind {
        AppQueryKind::UpdateList(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddShare {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub share_with: EntityUid,
    pub role: ShareRole,
}

impl From<AddShare> for AppQueryKind {
    fn from(v: AddShare) -> AppQueryKind {
        AppQueryKind::AddShare(v)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ShareRole {
    Reader,
    Editor,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteShare {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub unshare_with: EntityUid,
    pub role: ShareRole,
}

impl From<DeleteShare> for AppQueryKind {
    fn from(v: DeleteShare) -> AppQueryKind {
        AppQueryKind::DeleteShare(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteList {
    pub uid: EntityUid,
    pub list: EntityUid,
}

impl From<DeleteList> for AppQueryKind {
    fn from(v: DeleteList) -> AppQueryKind {
        AppQueryKind::DeleteList(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetLists {
    pub uid: EntityUid,
}

impl From<GetLists> for AppQueryKind {
    fn from(v: GetLists) -> AppQueryKind {
        AppQueryKind::GetLists(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTask {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub task: i64,
    pub description: Option<String>,
    pub state: Option<TaskState>,
}

impl From<UpdateTask> for AppQueryKind {
    fn from(v: UpdateTask) -> AppQueryKind {
        AppQueryKind::UpdateTask(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTask {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub description: String,
}

impl From<CreateTask> for AppQueryKind {
    fn from(v: CreateTask) -> AppQueryKind {
        AppQueryKind::CreateTask(v)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTask {
    pub uid: EntityUid,
    pub list: EntityUid,
    pub task: i64,
}

impl From<DeleteTask> for AppQueryKind {
    fn from(value: DeleteTask) -> Self {
        AppQueryKind::DeleteTask(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {
    message: &'static str,
}

impl Default for Empty {
    fn default() -> Self {
        Self { message: "ok" }
    }
}

pub async fn serve_api(chan: AppChannel, port: u16) {
    let filter = warp::path("api").and(
        // LIST CRUD
        (warp::path("list").and(
            (warp::path("get")
                .and(warp::get())
                .and(with_app(chan.clone()))
                .and(warp::query::query::<GetList>())
                .and_then(simple_query::<GetList, List>))
            .or(warp::path("create")
                .and(warp::post())
                .and(with_app(chan.clone()))
                .and(warp::body::json())
                .and_then(simple_query::<CreateList, EntityUid>))
            .or(warp::path("update")
                .and(warp::post())
                .and(with_app(chan.clone()))
                .and(warp::body::json())
                .and_then(simple_query::<UpdateList, Empty>))
            .or(warp::path("delete")
                .and(warp::delete())
                .and(with_app(chan.clone()))
                .and(warp::body::json())
                .and_then(simple_query::<DeleteList, Empty>)),
        ))
        .or(
            // Task CRUD
            warp::path("task").and(
                (warp::path("create")
                    .and(warp::post())
                    .and(with_app(chan.clone()))
                    .and(warp::body::json())
                    .and_then(simple_query::<CreateTask, i64>))
                .or(warp::path("update")
                    .and(warp::post())
                    .and(with_app(chan.clone()))
                    .and(warp::body::json())
                    .and_then(simple_query::<UpdateTask, Empty>))
                .or(warp::path("delete")
                    .and(warp::delete())
                    .and(with_app(chan.clone()))
                    .and(warp::body::json())
                    .and_then(simple_query::<DeleteTask, Empty>)),
            ),
        )
        .or(warp::path("lists")
            .and(warp::path("get"))
            .and(with_app(chan.clone()))
            .and(warp::query::query::<GetLists>())
            .and_then(simple_query::<GetLists, Lists>))
        .or(warp::path("share").and(
            (warp::post()
                .and(with_app(chan.clone()))
                .and(warp::body::json())
                .and_then(simple_query::<AddShare, Empty>))
            .or(warp::delete()
                .and(with_app(chan.clone()))
                .and(warp::body::json())
                .and_then(simple_query::<DeleteShare, Empty>)),
        )),
    );

    let s = warp::serve(filter);
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    s.run(socket).await
}

pub fn with_app(
    chan: AppChannel,
) -> impl Filter<Extract = (AppChannel,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || chan.clone())
}

#[derive(Serialize)]
struct ErrorMsg {
    #[serde(serialize_with = "serialize_error")]
    error: Error,
}

fn serialize_error<S>(e: &Error, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{}", e))
}

fn respond(msg: Result<impl Serialize, Error>) -> impl warp::Reply {
    match msg {
        Ok(msg) => Ok(serde_json::to_string(&msg).unwrap()),
        Err(error) => Ok(serde_json::to_string(&ErrorMsg { error }).unwrap()),
    }
}

pub async fn simple_query<I, R>(
    app: mpsc::Sender<AppQuery>,
    q: I,
) -> Result<impl warp::Reply, warp::Rejection>
where
    I: Into<AppQueryKind>,
    AppResponse: TryInto<R, Error = Error>,
    R: Serialize,
{
    let result = simple_query_inner::<R>(app, q).await;
    Ok(respond(result))
}

pub async fn simple_query_inner<R>(
    app: mpsc::Sender<AppQuery>,
    q: impl Into<AppQueryKind>,
) -> Result<R, Error>
where
    AppResponse: TryInto<R, Error = Error>,
    R: Serialize,
{
    let (send, recv) = oneshot::channel();
    let kind = q.into();
    let q = AppQuery::new(kind, send);
    app.send(q).await?;
    let resp = recv.await??;
    let resp = resp.try_into()?;
    Ok(resp)
}
