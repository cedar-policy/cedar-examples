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

mod api;
mod context;
mod entitystore;
mod objects;
mod policy_store;
mod policy_watcher;
mod timebox;
mod util;

use context::AppContext;
use std::num::ParseIntError;
use thiserror::Error;
use tracing::Level;

#[tokio::main]
async fn main() {
    init_logger();
    let app = AppContext::spawn(
        "./entities.json",
        "./tinytodo.cedarschema.json",
        "./policies.cedar",
    )
    .unwrap();
    let args = std::env::args().collect::<Vec<_>>();

    match get_port(&args) {
        Ok(port) => crate::api::serve_api(app, port).await,
        Err(e) => {
            eprintln!("Usage: {} <port>?\n{}", args[0], e);
            std::process::exit(1);
        }
    }
}

fn init_logger() {
    if let Ok(var) = std::env::var("RUST_LOG") {
        let level = match var.as_str() {
            "debug" => Level::DEBUG,
            "error" => Level::ERROR,
            "info" => Level::INFO,
            "trace" => Level::TRACE,
            "warn" => Level::WARN,
            _ => Level::INFO,
        };
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .pretty()
            .with_max_level(level)
            .finish();
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("Error setting up tracing: {e}");
        }
    }
}

#[derive(Debug, Clone, Error)]
enum ArgError {
    #[error("Couldn't parse port number. Expected a valid integer port number. {0}")]
    Parse(#[from] ParseIntError),
}

fn get_port(args: &[String]) -> Result<u16, ArgError> {
    let arg = args.get(1).map(String::as_str).unwrap_or("8080");
    let port: u16 = arg.parse()?;
    Ok(port)
}
