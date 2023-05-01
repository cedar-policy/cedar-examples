mod api;
mod context;
mod entitystore;
mod objects;
mod util;

use context::AppContext;
use entitystore::EntityStore;
use objects::{Team, UserOrTeam};
use std::io::Write;
use std::num::ParseIntError;
use thiserror::Error;

use crate::objects::User;

const APPLICATION: &str = r#"Application::"TinyTodo""#;

#[tokio::main]
async fn main() {
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

fn generate_input_data() -> std::io::Result<()> {
    let mut f = std::fs::File::create("./output.json")?;
    let mut store = EntityStore::default();
    teams(&mut store);
    users(&mut store);
    let src = serde_json::to_string(&store).unwrap();
    write!(f, "{src}")?;
    Ok(())
}

fn users(store: &mut EntityStore) {
    let users = [
        ("aaron", vec!["interns"]),
        ("andrew", vec!["admin", "temp"]),
        ("emina", vec!["admin"]),
        ("kesha", vec!["temp"]),
    ];

    for (user, parents) in users {
        let euid = format!(r#"User::"{}""#, user).parse().unwrap();
        let mut user = User::new(euid);
        for team in parents {
            let euid = format!(r#"Team::"{}""#, team).parse().unwrap();
            user.add_parent(euid);
        }
        store.insert_user(user);
    }
}

fn teams(store: &mut EntityStore) {
    let teams = [
        ("Admin", vec![]),
        ("interns", vec!["temp"]),
        ("temp", vec![]),
    ];
    for (team, parents) in teams {
        let euid = format!(r#"Team::"{}""#, team).parse().unwrap();
        let mut team = Team::new(euid);
        for parent in parents {
            let euid = format!(r#"Team::"{}""#, parent).parse().unwrap();
            team.add_parent(euid);
        }
        store.insert_team(team);
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
