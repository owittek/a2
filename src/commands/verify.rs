use serenity::{builder::CreateApplicationCommand, model::prelude::Connection};
use tracing::info;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("verify")
        .description("Verify as a contributor")
}

pub fn run(connections: Vec<Connection>) -> String {
    dbg!(&connections);
    let gh = &connections
        .iter()
        .find(|connection| connection.kind == "github");
    if let Some(gh) = gh {
        let res = "Found Github connection: ".to_owned() + &gh.name;
        info!(res);
        res
    } else {
        String::from("No GitHub connection found")
    }
}
