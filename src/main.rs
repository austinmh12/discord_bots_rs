#[allow(unused)]
#[allow(unused_imports)]

#[macro_use]
extern crate lazy_static;
extern crate base64;

use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use dotenv;

use serenity::{async_trait, model::channel::{Message}, framework::standard::{CommandOptions, Reason}};
use serenity::client::{Client, Context, EventHandler};
use serenity::model::{
	gateway::{
		Ready,
		GatewayIntents
	},
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

use commands::{
	poketcg::*,
	poketcg::player::*,
	poketcg::sets::*,
	poketcg::store::*
};

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
	trade_main,
	game_corner_main,
	upgrades_main,
	player_main,
	quiz_command,
	lightmode_command,
	binder_main,
	deck_main,
	decks_command,
)]
struct PokeTCG;

#[check]
#[name="BotTest"]
async fn owner_check(_: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> Result<(), Reason> {
	if msg.guild_id.unwrap() != 655509540543922217 {
		return Err(Reason::User("Not the bot testing server.".to_string()));
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
			let ctx1 = Arc::clone(&ctx);
			tokio::spawn(async move {
				loop {
					commands::poketcg::refresh_dailys(Arc::clone(&ctx1)).await;
					tokio::time::sleep(Duration::from_secs(60)).await;
				}
			});
			let ctx2 = Arc::clone(&ctx);
			tokio::spawn(async move {
				loop {
					commands::poketcg::refresh_card_prices(Arc::clone(&ctx2)).await;
					tokio::time::sleep(Duration::from_secs(3600)).await;
				}
			});
		}
	}

	// Here for getting custom emoji IDs
	// async fn reaction_add(&self, _ctx: Context, reaction: serenity::model::channel::Reaction) {
	// 	match reaction.emoji {
	// 		serenity::model::channel::ReactionType::Custom {animated: _, id: y, name: Some(_)} => println!("{}", y.0),
	// 		serenity::model::channel::ReactionType::Unicode(s) => println!("{}", s),
	// 		_ => ()
	// 	}
	// }
}

#[tokio::main]
async fn main() {
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("."))
		.group(&POKETCG_GROUP);

	dotenv::dotenv().ok();
	// Configure the client with the discord token. Make sure one is commented out.
	let token = dotenv::var("BOTTOKEN").expect("Expected a token in the environment");
	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::GUILD_MESSAGE_REACTIONS
		| GatewayIntents::MESSAGE_CONTENT;

	let handler = Handler {
		is_loop_running: AtomicBool::new(false),
	};

	// Create a new instance of the client logging in as the bot. This will automatically
	// prepend your bot token with "Bot ", which is required by discord.
	let mut client = Client::builder(&token, intents)
		.framework(framework)
		.event_handler(handler)
		.await
		.expect("Error creating client");

	// Finally start a shard and listen for events.
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}