#[macro_use]
extern crate lazy_static;

mod commands;

use dotenv;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler, bridge::gateway::GatewayIntents};
use serenity::model::{
	channel::{Reaction, ReactionType},
	gateway::Ready,
};
use serenity::framework::standard::{
    StandardFramework,
    macros::{
        group,
    },
};

use commands::{meme::*, youtube::*};

#[group]
#[commands(sheesh, amogus, blue)]
struct Meme;

#[group]
#[commands(subscribe, unsubscribe, subscriptions)]
struct YouTube;

struct Handler {
	database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Handler {
	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}

	async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
		if reaction.emoji.unicode_eq("✅") {
			let _ = reaction.message(&ctx).await.unwrap()
				.react(&ctx, ReactionType::try_from("<:backchk:799333634263613440>").unwrap()).await;
		}
	}
}

#[tokio::main]
async fn main() {
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("."))
		.group(&MEME_GROUP)
		.group(&YOUTUBE_GROUP);

	dotenv::dotenv().ok();
	// Configure the client with the discord token. Make sure one is commented out.
	// let token = dotenv::var("AUSTINTOKEN").expect("Expected a token in the environment");
	let token = dotenv::var("TESTBOT").expect("Expected a token in the environment");

	// Initiate database connection, creating the file if needed
	let database = sqlx::sqlite::SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(
			sqlx::sqlite::SqliteConnectOptions::new()
				.filename("bot.db")
				.create_if_missing(true),
		)
		.await
		.expect("Couldn't connect to database");
	
	// Run the migrations to update the schema to the latest version
	sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

	let handler = Handler {
		database
	};

	// Create a new instance of the client logging in as the bot. This will automatically
	// prepend your bot token with "Bot ", which is required by discord.
	let mut client = Client::builder(&token)
		.event_handler(handler)
		.intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS | GatewayIntents::default())
		.framework(framework)
		.await
		.expect("Error creating client");

	// Finally start a shard and listen for events.
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}