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

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use cedar_policy::{ParseErrors, PolicySet, Schema, SchemaError, ValidationError, Validator};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

use crate::context::{AppQuery, AppQueryKind};

#[derive(Debug, Clone)]
struct PolicySetWatcher {
    policy_set: PathBuf,
    schema: PathBuf,
    tx: Sender<AppQuery>,
}

type Result<A> = std::result::Result<A, Error>;

#[derive(Debug, Error)]
enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Errors parsing policy set: {0}")]
    ParsePolicies(#[from] ParseErrors),
    #[error("Errors parsing schema: {0}")]
    ParseSchema(#[from] SchemaError),
    #[error("Errors validating policy set: {0}")]
    Validation(String),
    #[error("Error sending to app processor: {0}")]
    McspChan(#[from] tokio::sync::mpsc::error::SendError<AppQuery>),
    #[error("Error receiving response from oneshot channel: {0}")]
    OneShot(#[from] tokio::sync::oneshot::error::RecvError),
}

#[derive(Debug)]
struct ValidationErrors<'a>(Vec<&'a ValidationError<'a>>);

impl<'a> Display for ValidationErrors<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in self.0.iter() {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error {
    pub fn validation<'a>(v: impl Iterator<Item = &'a ValidationError<'a>>) -> Self {
        Self::Validation(ValidationErrors(v.collect()).to_string())
    }
}

pub async fn spawn_watcher(
    policy_set: impl AsRef<Path>,
    schema: impl AsRef<Path>,
    tx: Sender<AppQuery>,
) {
    let w = PolicySetWatcher {
        policy_set: PathBuf::from(policy_set.as_ref()),
        schema: PathBuf::from(schema.as_ref()),
        tx,
    };
    tokio::spawn(async move { watcher_supervisor(w).await });
}

// This supervises the watcher task, reporting any errors and respawning the watcher
async fn watcher_supervisor(w: PolicySetWatcher) {
    loop {
        let cloned = w.clone();
        let handle = tokio::spawn(async { watcher(cloned).await });
        match handle.await {
            Ok(f) => match f {
                Ok(a) => match a {},
                Err(e) => debug!("Policy Set File Watcher died due to: {e}, respawning..."),
            },
            Err(e) => error!("Join Error: {e}"),
        }
    }
}

enum Empty {}

async fn watcher(w: PolicySetWatcher) -> Result<Empty> {
    let mut last_modified = get_last_modified(&w.policy_set).await?;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let time = get_last_modified(&w.policy_set).await?;
        if time != last_modified {
            last_modified = time;
            match attempt_policy_reload(&w).await {
                Ok(policies) => {
                    send_query(policies, &w.tx).await?;
                }
                Err(e) => error!("Error reloading policies: {e}"),
            };
        }
    }
}

async fn send_query(p: PolicySet, tx: &Sender<AppQuery>) -> Result<()> {
    let (send, recv) = tokio::sync::oneshot::channel();
    let query = AppQuery::new(AppQueryKind::UpdatePolicySet(p), send);
    tx.send(query).await?;
    let _ = recv.await?;
    Ok(())
}

async fn attempt_policy_reload(w: &PolicySetWatcher) -> Result<PolicySet> {
    let policies: PolicySet = tokio::fs::read_to_string(&w.policy_set).await?.parse()?;
    let schema: Schema = tokio::fs::read_to_string(&w.schema).await?.parse()?;
    let validator = Validator::new(schema);
    let results = validator.validate(&policies, cedar_policy::ValidationMode::Strict);
    if results.validation_passed() {
        Ok(policies)
    } else {
        Err(Error::validation(results.validation_errors()))
    }
}

async fn get_last_modified(path: &Path) -> std::io::Result<SystemTime> {
    let metadata = tokio::fs::metadata(path).await?;
    metadata.modified()
}
