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
	time::Duration as StdDuration,
	collections::HashMap, fmt::Write
};
use tokio::sync::RwLock;
use dotenv;

use serenity::{async_trait, model::channel::{Message}, framework::standard::{CommandOptions, Reason}, prelude::*};
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
use chrono::{DateTime, Utc, Duration};
use indicatif::*;
use rand::prelude::*;

mod commands;

use commands::{
	poketcg::{
		*,
		player::*,
		sets::*,
		store::*,
		binder::*,
		slot::*,
		quiz::*,
		decks::*,
		card::Card
	}
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

struct Cache;

impl TypeMapKey for Cache {
	type Value = Arc<RwLock<HashMap<String, CardCache>>>;
}

#[derive(Debug, Clone)]
pub struct CardCache {
	pub card: Card,
	pub next_update: DateTime<Utc>,
	pub last_accessed: DateTime<Utc>
}

impl CardCache {
	fn new(card: Card) -> Self {
		let hour_range = (0..23).collect::<Vec<i64>>();
		let rand_hours = hour_range.choose(&mut rand::thread_rng()).unwrap().clone();

		Self {
			card,
			next_update: Utc::now() + Duration::hours(rand_hours),
			last_accessed: Utc::now()
		}
	}
}

struct Handler {
	is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _ctx: Context, ready: Ready) {
		let sets = get_sets().await;
		let pb = ProgressBar::new(sets.len() as u64);
		pb.set_style(
			ProgressStyle::default_bar()
			.template("[{elapsed_precise}] [{bar:30.cyan/blue}] [{pos}/{len} (ETA {eta})] {msg}")
			.unwrap()
			.progress_chars("=> ")
		);
		for set in sets {
			// This ensures that all the rare/rainbow cards are in the cache before starting.
			pb.set_message(format!("Fetching cards for {}", &set.name));
			_ = card::get_cards_with_query(&_ctx, &format!("set.id:{} AND -rarity:Common AND -rarity:Uncommon AND -rarity:Promo", set.set_id)).await;
			pb.inc(1);
		}
		pb.finish_with_message("Fetched all the rare and rainbow cards.");
		let ctx = Arc::new(_ctx);
		
		if !self.is_loop_running.load(Ordering::Relaxed) {
			let ctx1 = Arc::clone(&ctx);
			tokio::spawn(async move {
				loop {
					commands::poketcg::refresh_dailys(Arc::clone(&ctx1)).await;
					tokio::time::sleep(StdDuration::from_secs(60)).await;
				}
			});
			let ctx2 = Arc::clone(&ctx);
			tokio::spawn(async move {
				loop {
					commands::poketcg::refresh_card_prices(Arc::clone(&ctx2)).await;
					tokio::time::sleep(StdDuration::from_secs(3600)).await;
				}
			});
			let ctx3 = Arc::clone(&ctx);
			tokio::spawn(async move {
				loop {
					commands::poketcg::check_daily_streaks(Arc::clone(&ctx3)).await;
					tokio::time::sleep(StdDuration::from_secs(1800)).await;
				}
			});
		}

		println!("{} is connected and ready!", ready.user.name);
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

	{
		let mut cache = client.data.write().await;
		cache.insert::<Cache>(Arc::new(RwLock::new(HashMap::default())));
	}

	// Finally start a shard and listen for events.
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}