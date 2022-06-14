use std::{
	collections::HashMap,
	cmp::Ordering
};

use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		Document,
		oid::ObjectId,
	}, 
	Collection
};
use chrono::{
	DateTime, 
	Utc, Local,
};
use futures::stream::{TryStreamExt};
use serenity::{
	framework::{
		standard::{
			macros::{
				command
			},
			Args,
			CommandResult
		},
	},
	builder::{
		CreateEmbed
	},
	model::{
		channel::{
			Message,
		},
	},
	utils::{
		Colour
	},
	prelude::*
};

use crate::{
	commands::get_client
};

use super::{
	PaginateEmbed,
	upgrade::Upgrade,
	binder::Binder,
	RARITY_ORDER,
	player_card,
	card_paginated_embeds,
	timers,
	HasSet,
	Idable
};

fn def_10() -> i64 {
	10
}

fn def_0() -> i64 {
	0
}

fn def_upgrade() -> Upgrade {
	Upgrade::new()
}

fn def_false() -> bool {
	false
}

fn def_binder() -> Binder {
	Binder::empty()
}

fn def_empty_vec_str() -> Vec<String> {
	vec![]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub discord_id: i64,
	pub cash: f64,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub daily_reset: DateTime<Utc>,
	pub packs: HashMap<String, i64>,
	pub packs_opened: i64,
	pub packs_bought: i64,
	pub total_cash: f64,
	pub cards: HashMap<String, i64>,
	pub total_cards: i64,
	pub cards_sold: i64,
	pub daily_packs: i64,
	pub quiz_questions: i64,
	pub current_multiplier: i64,
	pub quiz_correct: i64,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub quiz_reset: DateTime<Utc>,
	pub savelist: Vec<String>,
	pub perm_multiplier: i64,
	#[serde(default = "def_10")]
	pub daily_slots: i64,
	#[serde(default = "def_0")]
	pub slots_rolled: i64,
	#[serde(default = "def_0")]
	pub jackpots: i64,
	#[serde(default = "def_0")]
	pub boofs: i64,
	#[serde(default = "def_0")]
	pub tokens: i64,
	#[serde(default = "def_0")]
	pub total_tokens: i64,
	#[serde(default = "def_upgrade")]
	pub upgrades: Upgrade,
	#[serde(default = "def_false")]
	pub light_mode: bool,
	#[serde(default = "def_binder")]
	pub current_binder: Binder,
	#[serde(default = "def_empty_vec_str")]
	pub completed_binders: Vec<String>
}

impl Player {
	fn new_from_discord_id(discord_id: i64) -> Self {
		Self {
			id: None,
			discord_id,
			cash: 25.0,
			daily_reset: Utc::now(),
			packs: HashMap::new(),
			packs_opened: 0,
			packs_bought: 0,
			total_cash: 25.0,
			cards: HashMap::new(),
			total_cards: 0,
			cards_sold: 0,
			daily_packs: 50,
			quiz_questions: 5,
			current_multiplier: 1,
			quiz_correct: 0,
			quiz_reset: Utc::now(),
			savelist: vec![],
			perm_multiplier: 50,
			daily_slots: 10,
			slots_rolled: 0,
			jackpots: 0,
			boofs: 0,
			tokens: 0,
			total_tokens: 0,
			upgrades: Upgrade::new(),
			light_mode: false,
			current_binder: Binder::empty(),
			completed_binders: vec![],
		}
	}
}

impl PaginateEmbed for Player {
	fn embed(&self) -> CreateEmbed {
		let quiz_reset_local: DateTime<Local> = DateTime::from(self.quiz_reset);
		let daily_reset_local: DateTime<Local> = DateTime::from(self.daily_reset);
		let mut desc = format!("**Wallet:** ${:.2} | **Total Earned:** ${:.2}\n\n", &self.cash, &self.total_cash);
		desc.push_str(&format!("**Current Packs:** {}\n", self.packs.values().map(|v| v.clone() as i32).sum::<i32>()));
		desc.push_str(&format!("**Opened Packs:** {} | **Bought Packs:** {}\n\n", &self.packs_opened, &self.packs_bought));
		desc.push_str(&format!("**Total Cards:** {} | **Cards Sold:** {}\n\n", &self.total_cards, &self.cards_sold));
		desc.push_str(&format!("**Slot Rolls:** {} | **Slots Rolled:** {}\n", &self.daily_slots, &self.slots_rolled));
		desc.push_str(&format!("**Tokens:** {} | **Total Tokens:** {}\n", &self.tokens, &self.total_tokens));
		desc.push_str(&format!("**Jackpots:** {} | **Boofs:** {}\n\n", &self.jackpots, &self.boofs));
		desc.push_str(&format!("**Quiz Questions Remaining:** {}\n", &self.quiz_questions));
		desc.push_str(&format!("**Quiz Questions Answered:** {}\n\n", &self.quiz_correct));
		match self.current_binder.set.as_str() {
			"" => desc.push_str(&format!("**Current Binder:** None! | **Completed Binders:** {}\n\n", &self.completed_binders.len())),
			_ => desc.push_str(&format!("**Current Binder:** {} | **Completed Binders:** {}\n\n", &self.current_binder.set, &self.completed_binders.len()))
		}
		desc.push_str(&format!("Quiz resets at **{}**\n", quiz_reset_local.format("%m/%d %H:%M")));
		desc.push_str(&format!("Daily reset at **{}**", daily_reset_local.format("%m/%d %H:%M")));
		let mut ret = CreateEmbed::default();
		ret
			.description(desc)
			.colour(Colour::from_rgb(255, 50, 20));

		ret
	}
}

async fn get_player_collection() -> Collection<Player> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Player>("players");

	collection
}

// Database functions
pub async fn get_players() -> Vec<Player> { // Will change to Player
	let player_collection = get_player_collection().await;
	let players = player_collection
		.find(None, None)
		.await
		.unwrap()
		.try_collect::<Vec<Player>>()
		.await
		.unwrap();

	players
}

pub async fn get_player(discord_id: u64) -> Player { // Will change to Player
	let discord_id = discord_id as i64;
	let player_collection = get_player_collection().await;
	let player = player_collection
		.find_one(doc! { "discord_id": discord_id }, None)
		.await
		.unwrap();
	match player {
		Some(x) => return x,
		None => return add_player(discord_id).await
	}
}

async fn add_player(discord_id: i64) -> Player {
	let ret = Player::new_from_discord_id(discord_id);
	let player_collection = get_player_collection().await;
	player_collection
		.insert_one(&ret, None)
		.await
		.unwrap();
	
	ret
}

pub async fn update_player(player: &Player, update: Document) {
	let player_collection = get_player_collection().await;
	player_collection
		.update_one(
			doc! {"_id": &player.id.unwrap() }, 
			update, 
			None)
		.await
		.unwrap();
}

// COMMANDS
#[command("my")]
#[sub_commands(my_cards, my_packs, my_stats, my_upgrades)]
async fn my_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available my commands:
	**.my cards [sort_by - Default: name]** to view your cards.
	**.my packs** to view your packs.
	**.my stats** to view your stats.
	**.my upgrades** to view your upgrades.";
	let _ = get_player(msg.author.id.0).await;
	msg.reply(&ctx.http, content).await?;

	Ok(())
}

#[command("cards")]
#[aliases("c")]
async fn my_cards(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let sorting = match args.find::<String>() {
		Ok(x) => x.to_lowercase(),
		Err(_) => String::from("name")
	};
	let player = get_player(msg.author.id.0).await;
	let mut cards = player_card::player_cards(player.cards.clone()).await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "You have no cards!").await?;
		return Ok(());
	} else {
		match sorting.replace("-", "").as_str() {
			"id" => cards.sort_by(|c1, c2| {
				if c1.set().id() == c2.set().id() {
					let c1_num = c1.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);
					let c2_num = c2.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);
	
					c1_num.cmp(&c2_num)
				} else {
					c1.id().cmp(&c2.id())
				}
			}),
			"amount" => cards.sort_by(|c1, c2| c2.amount.cmp(&c1.amount)),
			"price" => cards.sort_by(|c1, c2| {
				if c1.card.price < c2.card.price {
					Ordering::Greater
				} else if c1.card.price == c2.card.price {
					Ordering::Equal
				} else {
					Ordering::Less
				}
			}),
			"rare" => cards.sort_by(|c1, c2| {
				let c1_rare_pos = RARITY_ORDER.iter().position(|r| &c1.card.rarity == r).unwrap_or(999);
				let c2_rare_pos = RARITY_ORDER.iter().position(|r| &c2.card.rarity == r).unwrap_or(999);

				c1_rare_pos.cmp(&c2_rare_pos)
			}),
			_ => cards.sort_by(|c1, c2| c1.card.name.cmp(&c2.card.name)),
		}
		if sorting.contains("-") {
			cards.reverse();
		}
		card_paginated_embeds(ctx, msg, cards, player).await?;
	}

	Ok(())
}

#[command("packs")]
#[aliases("p")]
async fn my_packs(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let timer = timers::get_timer().await;
	let mut desc = format!("You have **{}** packs left to open today\n", player.daily_packs);
	desc.push_str("Use **.(op)enpack <set_id> [amount]** to open packs\n");
	for (set_id, amount) in player.packs.iter() {
		desc.push_str(&format!("**{}** - {}\n", set_id, amount));
	}
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.title("Your packs")
					.description(&desc)
					.colour(Colour::from_rgb(255, 50, 20))
					.footer(|f| {
						let local_timer: DateTime<Local> = DateTime::from(timer.pack_reset);

						f.text(&format!("Resets {}", local_timer.format("%h %d %H:%M")))
					})
			})
		})
		.await?;

	Ok(())
}

#[command("stats")]
#[aliases("s")]
async fn my_stats(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let nickname = match msg.author_nick(ctx).await {
		Some(x) => x,
		None => msg.author.name.clone()
	};
	let avatar_url = msg.author.avatar_url().unwrap();
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut e = player.embed();
			
			e
				.title(nickname)
				.thumbnail(avatar_url);
			m.set_embed(e);

			m
		})
			.await?;

	Ok(())
}

#[command("upgrades")]
#[aliases("ups")]
async fn my_upgrades(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let nickname = match msg.author_nick(ctx).await {
		Some(x) => x,
		None => msg.author.name.clone()
	};
	let avatar_url = msg.author.avatar_url().unwrap();
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.title(nickname)
					.thumbnail(avatar_url)
					.description(player.upgrades.desc())
					.colour(Colour::from_rgb(255, 50, 20))
			});

			m
		})
			.await?;

	Ok(())
}

#[command("player")]
#[aliases("pl")]
#[sub_commands(player_cards, player_packs, player_stats, player_upgrades)]
async fn player_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available player commands:
	**.player cards [sort_by - Default: name]** to view a player's cards.
	**.player packs** to view a player's packs.
	**.player stats** to view a player's stats.
	**.player upgrades** to view a player's upgrades";
	let _ = get_player(msg.author.id.0).await;
	msg.reply(&ctx.http, content).await?;

	Ok(())
}

#[command("cards")]
#[aliases("c")]
async fn player_cards(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let player_mention = msg.mentions.iter().nth(0);
	match player_mention {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You didn't mention a player.").await?;
			return Ok(());
		}
	}
	let player_mention = player_mention.unwrap();
	let player = get_player(player_mention.id.0).await;
	args.advance();
	let sorting = match args.single::<String>() {
		Ok(x) => x.to_lowercase(),
		Err(_) => String::from("name")
	};
	let mut cards = player_card::player_cards(player.cards.clone()).await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "You have no cards!").await?;
	} else {
		match sorting.replace("-", "").as_str() {
			"id" => cards.sort_by(|c1, c2| {
				if c1.set().id() == c2.set().id() {
					let c1_num = c1.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);
					let c2_num = c2.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);
	
					c1_num.cmp(&c2_num)
				} else {
					c1.id().cmp(&c2.id())
				}
			}),
			"amount" => cards.sort_by(|c1, c2| c2.amount.cmp(&c1.amount)),
			"price" => cards.sort_by(|c1, c2| {
				if c1.card.price < c2.card.price {
					Ordering::Greater
				} else if c1.card.price == c2.card.price {
					Ordering::Equal
				} else {
					Ordering::Less
				}
			}),
			"rare" => cards.sort_by(|c1, c2| {
				let c1_rare_pos = RARITY_ORDER.iter().position(|r| &c1.card.rarity == r).unwrap_or(999);
				let c2_rare_pos = RARITY_ORDER.iter().position(|r| &c2.card.rarity == r).unwrap_or(999);

				c1_rare_pos.cmp(&c2_rare_pos)
			}),
			_ => cards.sort_by(|c1, c2| c1.card.name.cmp(&c2.card.name)),
		}
		if sorting.contains("-") {
			cards.reverse();
		}
		card_paginated_embeds(ctx, msg, cards, player).await?;
	}

	Ok(())
}

#[command("packs")]
#[aliases("p")]
async fn player_packs(ctx: &Context, msg: &Message) -> CommandResult {
	let player_mention = msg.mentions.iter().nth(0);
	match player_mention {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You didn't mention a player.").await?;
			return Ok(());
		}
	}
	let player_mention = player_mention.unwrap();
	let player = get_player(player_mention.id.0).await;
	let nickname = match player_mention.nick_in(&ctx.http, msg.guild_id.unwrap()).await {
		Some(x) => x,
		None => player_mention.name.clone()
	};
	let timer = timers::get_timer().await;
	let mut desc = format!("{} has **{}** packs left to open today\n", nickname, player.daily_packs);
	for (set_id, amount) in player.packs.iter() {
		desc.push_str(&format!("**{}** - {}\n", set_id, amount));
	}
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.title(format!("{}'s packs", nickname))
					.description(&desc)
					.colour(Colour::from_rgb(255, 50, 20))
					.footer(|f| {
						let local_timer: DateTime<Local> = DateTime::from(timer.pack_reset);

						f.text(&format!("Resets {}", local_timer.format("%h %d %H:%M")))
					})
			})
		})
		.await?;

	Ok(())
}

#[command("stats")]
#[aliases("s")]
async fn player_stats(ctx: &Context, msg: &Message) -> CommandResult {
	let player_mention = msg.mentions.iter().nth(0);
	match player_mention {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You didn't mention a player.").await?;
			return Ok(());
		}
	}
	let player_mention = player_mention.unwrap();
	let player = get_player(player_mention.id.0).await;
	let nickname = match player_mention.nick_in(&ctx.http, msg.guild_id.unwrap()).await {
		Some(x) => x,
		None => player_mention.name.clone()
	};
	let avatar_url = player_mention.avatar_url().unwrap();
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut e = player.embed();
			
			e
				.title(nickname)
				.thumbnail(avatar_url);
			m.set_embed(e);

			m
		})
			.await?;

	Ok(())
}

#[command("upgrades")]
#[aliases("ups")]
async fn player_upgrades(ctx: &Context, msg: &Message) -> CommandResult {
	let player_mention = msg.mentions.iter().nth(0);
	match player_mention {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You didn't mention a player.").await?;
			return Ok(());
		}
	}
	let player_mention = player_mention.unwrap();
	let player = get_player(player_mention.id.0).await;
	let nickname = match player_mention.nick_in(&ctx.http, msg.guild_id.unwrap()).await {
		Some(x) => x,
		None => player_mention.name.clone()
	};
	let avatar_url = player_mention.avatar_url().unwrap();
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.title(nickname)
					.thumbnail(avatar_url)
					.description(player.upgrades.desc())
					.colour(Colour::from_rgb(255, 50, 20))
			});

			m
		})
			.await?;

	Ok(())
}