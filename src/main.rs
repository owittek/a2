mod commands;
use std::env;

use serenity::futures::future::join;

use serenity::async_trait;
use serenity::model::prelude::{
    Activity, GuildId, Interaction, InteractionResponseType, Message, Ready,
};
use serenity::prelude::*;
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "verify" => {
                    let connections = ctx.http.get_user_connections().await;
                    if let Err(why) = &connections {
                        let message = "Failed to get User connections";
                        error!("{}: {}", message, why);
                    }
                    commands::verify::run(connections.unwrap())
                }
                _ => String::from("Unknown command"),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                error!("Failed to respond to slash command: {}", why);
            }
        };
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let set_status = ctx.online();
        let set_activity = ctx.set_activity(Activity::playing("with thy mother"));

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected a GUILD_ID in the environment")
                .parse()
                .expect("GUILD_ID must be a valid integer"),
        );

        if let Err(e) = GuildId::create_application_command(&guild_id, &ctx.http, |command| {
            commands::verify::register(command)
        })
        .await
        {
            error!("Failed to create verification command: {}", e);
        }

        // let verification_command =
        //     Command::create_global_application_command(&ctx.http, |command| {
        //         commands::verify::register(command)
        //     })
        //     .await;

        // match verification_command {
        //     Err(e) => error!("Failed to create verification command: {}", e),
        //     _ => info!("Successfully created verification command"),
        // }

        join(set_status, set_activity).await;
    }

    async fn message(&self, context: Context, msg: Message) {
        info!("Received {}", msg.content);
        if msg.content == "!messageme" {
            let res = msg
                .channel_id
                .send_message(&context, |m| m.content(format!("Hello {}!", msg.author)))
                .await;

            if let Err(why) = res {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
