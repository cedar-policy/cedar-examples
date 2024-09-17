use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::entity_graph::EntityGraph;
use crate::rego_requests::gdrive;
use crate::rego_requests::github::{GithubOpaInput, GithubRequest};
use crate::tinytodo_generator::{
    cedar::Request as TinyTodoCedarRequest, entities::Entities as TinyTodoGeneratedEntities,
    opa::to_opa,
};
use crate::ExampleApp;
use crate::SingleExecutionReport;
use cedar_policy_core::ast::Request;
use cedar_policy_core::authorizer::Decision;
use cedar_policy_core::entities::NoEntitiesSchema;
use cedar_policy_core::extensions::Extensions;
use log::warn;
use serde::Deserialize;
use serde_json::Value;

static GO_BINARY_PATH: &str = "rego-harness/build/bin/main";

pub struct RegoEngine<'a, T>
where
    T: EntityGraph,
{
    /// app object
    app: &'a ExampleApp,
    /// entity data
    entities: T,
}

impl<'a, T: EntityGraph> RegoEngine<'a, T> {
    pub fn new(
        app: &'a ExampleApp,
        entities: impl IntoIterator<Item = &'a cedar_policy_core::ast::Entity>,
    ) -> Self {
        Self {
            app,
            entities: T::from_iter(entities),
        }
    }

    pub fn execute(&self, requests: Vec<Request>) -> impl Iterator<Item = SingleExecutionReport> {
        match self.app.name {
            "tinytodo" => {
                let requests: Vec<_> = requests
                    .into_iter()
                    .map(TinyTodoCedarRequest::from)
                    .collect();
                let entities = TinyTodoGeneratedEntities::from_cedar_entities(self.entities.iter());
                let opa_json_payload = serde_json::to_value(to_opa(&entities, &requests)).unwrap();
                run_opa_requests(&opa_json_payload)
            }
            "github" => {
                let requests: Vec<_> = requests
                    .iter()
                    .map(|r| GithubRequest::new(r, &self.entities))
                    .collect();
                let json_payload = serde_json::to_value(GithubOpaInput::new(requests)).unwrap();
                run_opa_requests(&json_payload)
            }
            "gdrive" => {
                let closed = cedar_policy_core::entities::Entities::from_entities(
                    self.entities.iter(),
                    None::<&NoEntitiesSchema>,
                    cedar_policy_core::entities::TCComputation::ComputeNow,
                    Extensions::all_available(),
                )
                .unwrap();
                let requests: Vec<_> = requests
                    .iter()
                    .map(|r| gdrive::Request::new(r, &self.entities, &closed))
                    .collect();
                let json_payload = serde_json::to_value(gdrive::Input::new(requests)).unwrap();
                run_opa_requests(&json_payload)
            }
            appname => {
                warn!("Rego engine for {appname} is not yet implemented; not collecting data for Rego with {appname}");
                run_opa_requests(&serde_json::json!([]))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoTestOutput {
    #[serde(rename = "Decision")]
    decision: bool,
    #[serde(rename = "Dur")]
    dur_nanoseconds: u64,
}

impl From<GoTestOutput> for SingleExecutionReport {
    fn from(val: GoTestOutput) -> Self {
        let decision = if val.decision {
            Decision::Allow
        } else {
            Decision::Deny
        };
        SingleExecutionReport {
            dur: Duration::from_nanos(val.dur_nanoseconds),
            decision,
            errors: vec![],
            context_attrs: 0,
        }
    }
}

/// If requests is the empty list `[]`, then no requests will be run, and the
/// returned iterator will be empty
pub fn run_opa_requests(requests: &Value) -> impl Iterator<Item = SingleExecutionReport> {
    let json = serde_json::to_string(requests).unwrap();
    let test_outputs: Vec<GoTestOutput> = match json.as_str() {
        "[]" => vec![],
        json => {
            let process_output = run_process(json);
            serde_json::from_str::<Vec<GoTestOutput>>(&process_output).unwrap()
        }
    };
    test_outputs.into_iter().map(|result| result.into())
}

fn run_process(input: &str) -> String {
    let mut process_handle = Command::new(GO_BINARY_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = process_handle.stdin.as_mut().unwrap();
    writeln!(stdin, "{input}").unwrap();
    let output = process_handle.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}
