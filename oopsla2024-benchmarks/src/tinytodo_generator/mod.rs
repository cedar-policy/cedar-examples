pub mod cedar;
pub mod common;
pub mod constants;
pub mod entities;
pub mod list;
pub mod opa;
pub mod teams;
pub mod users;

use crate::RandomBytes;
use arbitrary::{Arbitrary, Unstructured};
use cedar::CedarOutput;
use constants::{MAX_TASKS, NUM_LISTS, NUM_REQUESTS, NUM_USERS, TEAM_DEPTH};
pub use opa::{to_opa, OpaOutput};
use serde::Serialize;

#[derive(Debug)]
pub struct TinyTodoConfig {
    tasks: usize,
    lists: usize,
    users: usize,
    requests: usize,
    team_depth: usize,
    pub cedar_only: bool,
}

impl Default for TinyTodoConfig {
    fn default() -> Self {
        Self {
            tasks: 20,
            lists: Default::default(),
            users: Default::default(),
            requests: Default::default(),
            team_depth: Default::default(),
            cedar_only: true,
        }
    }
}

impl TinyTodoConfig {
    pub fn apply_config(&self) {
        NUM_LISTS.with(|num_lists| *num_lists.borrow_mut() = self.lists);
        NUM_USERS.with(|num_lists| *num_lists.borrow_mut() = self.users);
        MAX_TASKS.with(|num_lists| *num_lists.borrow_mut() = self.tasks);
        NUM_REQUESTS.with(|num_lists| *num_lists.borrow_mut() = self.requests);
        TEAM_DEPTH.with(|team_depth| *team_depth.borrow_mut() = self.team_depth)
    }

    pub fn with(entities_size: usize, cedar_only: bool) -> Self {
        let team_depth = (entities_size as f64).log2().floor() as usize;
        TinyTodoConfig {
            tasks: entities_size,
            lists: entities_size,
            users: entities_size,
            requests: entities_size,
            team_depth,
            cedar_only,
        }
    }

    pub fn size(&self) -> usize {
        self.lists
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TinyTodoOutput {
    pub cedar: CedarOutput,
    pub opa: OpaOutput,
}

impl<'a> Arbitrary<'a> for TinyTodoOutput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let cedar: CedarOutput = u.arbitrary()?;
        let opa = to_opa(&cedar.entities, &cedar.requests);
        Ok(TinyTodoOutput { cedar, opa })
    }
}

pub fn generate_tinytodo_input(config: &TinyTodoConfig) -> (TinyTodoOutput, usize) {
    config.apply_config();
    let mut rbytes = RandomBytes::new();
    let mut size = 10240;
    loop {
        let mut u = rbytes.unstructured(size);
        match TinyTodoOutput::arbitrary(&mut u) {
            Ok(e) => {
                return (e, size);
            }
            Err(_) => {
                eprintln!("Exhausted randomness, retrying");
                size *= 2;
            }
        }
    }
}
