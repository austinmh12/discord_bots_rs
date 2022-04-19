#[allow(unused)]
#[allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use dotenv;

use serenity::{async_trait, model::channel::Message, framework::standard::{CommandOptions, Reason}};
use serenity::client::{Client, Context, EventHandler, bridge::gateway::GatewayIntents};
use serenity::model::{
	channel::{Reaction, ReactionType},
	gateway::Ready,
};
use serenity::framework::standard::{
    StandardFramework,
    macros::{
        group,
		check
    },
	Args,
};

mod commands;

use commands::{meme::*, youtube::*, poketcg::*};

#[group]
#[commands(sheesh, amogus, blue)]
struct Meme;

#[group]
#[commands(subscribe, unsubscribe, subscriptions, latest_video)]
struct YouTube;

#[group]
#[commands(
	search_main,
	sets_command,
	set_command,
	admin_main,
	my_main,
	store_main,
	daily_command,
	open_pack_command,
	sell_main,
	savelist_main,
)]
struct PokeTCG;

#[check]
#[name="Owner"]
async fn owner_check(_: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> Result<(), Reason> {
	if msg.author.id != 223616191246106624 {
		return Err(Reason::User("Not the owner.".to_string()));
	}

	Ok(())
}

struct Handler {
	is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _ctx: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);

		let ctx = Arc::new(_ctx);

		if !self.is_loop_running.load(Ordering::Relaxed) {
			let ctx1 = Arc::new(ctx);
			tokio::spawn(async move {
				loop {
					// commands::youtube::check_for_new_videos(Arc::clone(&ctx1)).await;
					commands::poketcg::refresh_daily_packs(Arc::clone(&ctx1)).await;
					tokio::time::sleep(Duration::from_secs(60)).await;
				}
			});
		}
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
		// .group(&YOUTUBE_GROUP)
		.group(&POKETCG_GROUP)
		;

	dotenv::dotenv().ok();
	// Configure the client with the discord token. Make sure one is commented out.
	let token = dotenv::var("BOTTOKEN").expect("Expected a token in the environment");

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
		is_loop_running: AtomicBool::new(false),
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