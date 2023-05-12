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
    path::{Path, PathBuf},
    time::Duration,
};

use notify::{
    event::{AccessKind, AccessMode},
    Config, Error, Event, EventKind, RecommendedWatcher, Watcher,
};
use tokio::sync::mpsc::Sender;
use tracing::{error, trace};

use crate::context::{AppQuery, AppQueryKind};

#[derive(Debug)]
pub struct PolicySetWatcher {
    watcher: RecommendedWatcher,
    path: PathBuf,
}

impl PolicySetWatcher {
    pub fn path(&'_ self) -> impl AsRef<Path> + '_ {
        &self.path
    }

    #[tracing::instrument]
    pub fn new(tx: Sender<AppQuery>, path: &Path) -> Self {
        let config = Config::default().with_poll_interval(Duration::from_secs(1));
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, Error>| match res {
                Ok(event) => {
                    trace!("Event: {:?}", event);
                    // This is less clean then I'd like, but modern editors seem to edit files in different ways.
                    // I've tested this w/ VSCode and Neovim.
                    // Closing a buffer in Neovim triggers the `Remove` event, require re-watching the file
                    // Closing a file in VSCode triggers the `Close` event.
                    match event.kind {
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            let (send, _recv) = tokio::sync::oneshot::channel();
                            let kind = AppQueryKind::UpdatePolicySet;
                            let q = AppQuery::new(kind, send);
                            tx.blocking_send(q).expect("Failed to send");
                        }
                        EventKind::Remove(_) => {
                            let (send, _recv) = tokio::sync::oneshot::channel();
                            let kind = AppQueryKind::UpdatePolicySet;
                            let q = AppQuery::new(kind, send);
                            tx.blocking_send(q).expect("Failed to send");
                            let (send, _recv) = tokio::sync::oneshot::channel();
                            let kind = AppQueryKind::ResetWatch;
                            let q = AppQuery::new(kind, send);
                            tx.blocking_send(q).expect("Failed to send");
                        }
                        _ => (),
                    }
                }
                Err(err) => error!("Error receiving filesystem event: {}", err),
            },
            config,
        )
        .expect("Failed to create watcher");

        let mut s = Self {
            watcher,
            path: PathBuf::from(path),
        };
        s.set_watch();
        s
    }

    #[tracing::instrument]
    pub fn set_watch(&mut self) {
        if let Err(e) = self
            .watcher
            .watch(self.path.as_ref(), notify::RecursiveMode::NonRecursive)
        {
            error!("Failed to set watch: {}", e);
        } else {
            trace!("Set watch");
        }
    }
}
