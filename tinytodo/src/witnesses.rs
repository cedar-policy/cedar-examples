use std::marker::PhantomData;

use cedar_policy::{Authorizer, Context, Decision, PolicySet, Request};

use crate::{
    context::{
        Error, Result, ACTION_CREATE_LIST, ACTION_CREATE_TASK, ACTION_DELETE_LIST,
        ACTION_DELETE_TASK, ACTION_EDIT_SHARE, ACTION_GET_LIST, ACTION_UPDATE_LIST,
        ACTION_UPDATE_TASK,
    },
    entitystore::SealedBundle,
    util::EntityUid,
};

pub struct AuthWitness<Action> {
    marker: PhantomData<Action>,
}

pub trait ReadList {}
pub trait WriteList {}
pub trait Delete {}
pub trait ReadUser {}
pub trait WriteUser {}
pub trait ReadTeam {}
pub trait WriteTeam {}
pub trait WriteTeamUser {}
pub trait ReadAll {}
pub trait CreateUser {}
pub trait CreateList {}
pub trait CreateTeam {}

struct InternalProof;
impl ReadAll for InternalProof {}

pub fn is_authorized<A: Action>(
    principal: &A::Principal,
    resource: &A::Resource,
    entities: SealedBundle,
    policies: &PolicySet,
) -> Result<AuthWitness<A>>
where
    A::Principal: AsRef<EntityUid>,
    A::Resource: AsRef<EntityUid>,
{
    let r = Request::new(
        Some(principal.as_ref().clone().into()),
        Some(A::action().clone().into()),
        Some(resource.as_ref().clone().into()),
        Context::empty(),
        None,
    )
    .unwrap();
    let entities = entities.unwrap(InternalProof);
    let response = Authorizer::new().is_authorized(&r, policies, &entities);
    match response.decision() {
        Decision::Allow => Ok(AuthWitness {
            marker: PhantomData,
        }),
        Decision::Deny => Err(Error::AuthDenied(response.diagnostics().clone())),
    }
}

pub trait Action {
    type Principal;
    type Resource;
    fn action() -> &'static EntityUid;
}

pub mod actions {
    use super::*;
    use crate::{
        context::ACTION_GET_LISTS,
        util::{ApplicationUid, ListUid, UserUid},
    };

    pub struct CreateList;
    impl Action for CreateList {
        type Principal = UserUid;
        type Resource = ApplicationUid;
        fn action() -> &'static EntityUid {
            &ACTION_CREATE_LIST
        }
    }

    impl super::CreateList for CreateList {}
    impl super::CreateTeam for CreateList {}

    pub struct GetList;
    impl Action for GetList {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_GET_LIST
        }
    }

    impl super::ReadList for GetList {}

    pub struct GetLists;
    impl Action for GetLists {
        type Principal = UserUid;
        type Resource = ApplicationUid;
        fn action() -> &'static EntityUid {
            &ACTION_GET_LISTS
        }
    }

    impl super::ReadAll for GetLists {}

    pub struct UpdateList;
    impl Action for UpdateList {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_UPDATE_LIST
        }
    }

    impl super::WriteList for UpdateList {}

    pub struct DeleteList;
    impl Action for DeleteList {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_DELETE_LIST
        }
    }

    impl super::Delete for DeleteList {}

    pub struct CreateTask;
    impl Action for CreateTask {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_CREATE_TASK
        }
    }

    impl super::WriteList for CreateTask {}

    pub struct UpdateTask;
    impl Action for UpdateTask {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_UPDATE_TASK
        }
    }

    impl super::WriteList for UpdateTask {}

    pub struct DeleteTask;
    impl Action for DeleteTask {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_DELETE_TASK
        }
    }

    impl super::WriteList for DeleteTask {}

    pub struct EditShare;
    impl Action for EditShare {
        type Principal = UserUid;
        type Resource = ListUid;
        fn action() -> &'static EntityUid {
            &ACTION_EDIT_SHARE
        }
    }

    impl super::WriteTeamUser for EditShare {}
    impl super::ReadList for EditShare {}
}
