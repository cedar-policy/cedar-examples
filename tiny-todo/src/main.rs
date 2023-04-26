mod api;
mod context;
mod entitystore;
mod objects;
mod util;

use context::AppContext;
use std::num::ParseIntError;
use thiserror::Error;

const APPLICATION: &str = r#"Application::"TinyTodo""#;

#[tokio::main]
async fn main() {
    let app = AppContext::spawn("./entities.json", "./schema.json", "./policies.cedar").unwrap();
    let args = std::env::args().collect::<Vec<_>>();

    match get_port(&args) {
        Ok(port) => crate::api::serve_api(app, port).await,
        Err(e) => {
            eprintln!("Usage: {} <port>?\n{}", args[0], e);
            std::process::exit(1);
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
