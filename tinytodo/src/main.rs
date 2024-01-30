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
mod util;

use context::AppContext;
use std::num::ParseIntError;
use thiserror::Error;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    init_logger();
    let (schema_path, policies_path) = if cfg!(feature = "use-templates") {
        (
            "./tinytodo-templates.cedarschema.json",
            "./policies-templates.cedar",
        )
    } else {
        ("./tinytodo.cedarschema.json", "./policies.cedar")
    };
    let app = match AppContext::spawn("./entities.json", schema_path, policies_path) {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to load entities, policies, or schema: {e}");
            std::process::exit(1);
        }
    };
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
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Error setting up tracing: {e}");
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
