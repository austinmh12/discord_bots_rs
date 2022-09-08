use std::collections::HashMap;
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
use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
		Document
	}, 
	Collection
};
use chrono::{
	TimeZone,
	DateTime, 
	Utc,
	Datelike,
	Duration,
	Local,
};
use rand::{
	seq::{
		SliceRandom
	},
	prelude::*
};
use crate::{
	sets::{
		Set,
		get_sets,
		get_set
	},
	card::{
		get_rare_cards_from_cache,
		get_rainbow_cards_from_cache,
		get_card,
	},
	player::{
		Player,
		get_player,
		update_player
	},
	commands::get_client,
	timers
};

use super::Idable;

const SLOT_OPTIONS: &'static [&str] = &[
	"7",
	"R",
	"Pikachu",
	"Slowpoke",
	"Magnemite",
	"Shellder",
	"Cherry",
];

lazy_static! {
	static ref SLOT_OPTION_IDS: HashMap<&'static str, i64> = {
		let mut m = HashMap::new();
		m.insert("7", 967522912242384906);
		m.insert("R", 967522912166903858);
		m.insert("Pikachu", 967522912196239510);
		m.insert("Slowpoke", 967522912275922995);
		m.insert("Magnemite", 967522912154296410);
		m.insert("Shellder", 967522912229793882);
		m.insert("Cherry", 967522912166871080);
		
		m
	};
}

static DEFAULT_WEIGHT: i32 = 10;
static FAVOURED_WEIGHT: i32 = 45;

pub struct Slot {
	pub rolls: Vec<SlotRoll>
}

impl Slot {
	pub fn new(amount: i64) -> Self {
		let mut rolls = vec![];
		for _ in 0..amount {
			let roll = SlotRoll::new();
			rolls.push(roll);
		}

		Self {
			rolls
		}
	}
}

pub struct SlotRoll {
	pub slot1: String,
	pub slot2: String,
	pub slot3: String
}

impl SlotRoll {
	pub fn new() -> Self {
		let slot1 = SLOT_OPTIONS
			.choose(&mut thread_rng()).unwrap().to_string();
		let mut slot_weights = vec![];
		for slot_option in SLOT_OPTIONS {
			if slot_option.to_string() == slot1 {
				slot_weights.push((slot_option, FAVOURED_WEIGHT));
			} else {
				slot_weights.push((slot_option, DEFAULT_WEIGHT));
			}
		}
		let slot2 = slot_weights
			.choose_weighted(&mut thread_rng(), |sw| sw.1)
			.unwrap()
			.0
			.to_string();
		let slot3 = if slot2 == slot1 {
			slot_weights
				.choose_weighted(&mut thread_rng(), |sw| sw.1)
				.unwrap()
				.0
				.to_string()
		} else {
			SLOT_OPTIONS.choose(&mut thread_rng()).unwrap().to_string()
		};

		Self {
			slot1,
			slot2,
			slot3
		}
	}

	pub fn reward(&self, upgrade_level: i64) -> i64 {
		let reward_mult = 1.0 + upgrade_level as f64 * 0.1;
		match (self.slot1.as_str(), self.slot2.as_str(), self.slot3.as_str()) {
			("7", "7", "7") => (500.0 * reward_mult) as i64,
			("R", "R", "R") => (200.0 * reward_mult) as i64,
			("Pikachu", "Pikachu", "Pikachu") => (120.0 * reward_mult) as i64,
			("Slowpoke", "Slowpoke", "Slowpoke") => (80.0 * reward_mult) as i64,
			("Magnemite", "Magnemite", "Magnemite") => (50.0 * reward_mult) as i64,
			("Shellder", "Shellder", "Shellder") => (30.0 * reward_mult) as i64,
			("Cherry", "Cherry", "Cherry") => (15.0 * reward_mult) as i64,
			("Cherry", "Cherry", _) | ("Cherry", _, "Cherry") | (_, "Cherry", "Cherry") => (5.0 * reward_mult) as i64,
			_ => 0
		}
	}

	pub fn reward_display(&self, upgrade_level: i64) -> String {
		let slot1_id = SLOT_OPTION_IDS.get(self.slot1.as_str()).unwrap();
		let slot2_id = SLOT_OPTION_IDS.get(self.slot2.as_str()).unwrap();
		let slot3_id = SLOT_OPTION_IDS.get(self.slot3.as_str()).unwrap();
		let reward = self.reward(upgrade_level);

		match reward {
			0 => format!("<:GameCorner{}:{}> <:GameCorner{}:{}> <:GameCorner{}:{}> Better luck next time!", self.slot1, slot1_id, self.slot2, slot2_id, self.slot3, slot3_id),
			_ => format!("<:GameCorner{}:{}> <:GameCorner{}:{}> <:GameCorner{}:{}> You won **{}** tokens!", self.slot1, slot1_id, self.slot2, slot2_id, self.slot3, slot3_id, reward)
		}

	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenShop {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub sets: Vec<String>,
	pub rare_card: String,
	pub rainbow_card: String,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub reset: DateTime<Utc>
}

impl TokenShop {
	async fn new(ctx: &Context) -> Self {
		let sets = get_sets().await;
		let mut weighted_sets = vec![];
		for set in sets {
			let weight = set.release_date.year() - 1998;
			weighted_sets.push((set, weight));
		}
		let store_sets = weighted_sets
			.choose_multiple_weighted(
				&mut thread_rng(),
				3,
				|ws| ws.1
			)
			.unwrap()
			.collect::<Vec<_>>()
			.iter()
			.map(|ws| ws.0.clone())
			.collect::<Vec<Set>>()
			.iter()
			.map(|s| s.id())
			.collect();
		let rare_cards = get_rare_cards_from_cache(ctx).await;
		let rainbows = get_rainbow_cards_from_cache(ctx).await;
		let rare_card = rare_cards
			.iter()
			.choose(&mut thread_rng())
			.unwrap()
			.clone()
			.id();
		let rainbow_card = rainbows
			.iter()
			.choose(&mut thread_rng())
			.unwrap()
			.clone()
			.id();
		let now = Utc::now() + Duration::days(1);

		Self {
			id: None,
			sets: store_sets,
			rare_card,
			rainbow_card,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	async fn update_shop(&self, ctx: &Context) -> Self {
		let tmp_tokenshop = TokenShop::new(ctx).await;
		let now = Utc::now() + Duration::days(1);

		Self {
			id: self.id,
			sets: tmp_tokenshop.sets,
			rare_card: tmp_tokenshop.rare_card,
			rainbow_card: tmp_tokenshop.rainbow_card,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	pub async fn embed_with_player(&self, ctx: &Context, player: Player) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		let mut desc = String::from("Welcome to the **Token Shop**! Here you can spend tokens for prized\n");
		desc.push_str(&format!("You have **{}** tokens\n", player.tokens));
		desc.push_str("Here are the prizes available today. To purchase one, use **.gamecorner tokenstore (b)uy <slot no> [amount - Default 1]**\n\n");
		let discount = 1.0 + player.upgrades.tokenshop_discount as f64 * 0.05;
		for (i, set_id) in self.sets.iter().enumerate() {
			let num = i + 1;
			let set = get_set(set_id).await.unwrap();
			desc.push_str(&format!("**{}:** {} (_{}_) - {} tokens\n", num, set.name, set.id(), (to_tokens(set.pack_price()) as f64 / discount) as i64));
		}
		let rare_card = get_card(ctx, &self.rare_card).await;
		desc.push_str(&format!("**4:** {} (_{}_) - {} tokens\n", rare_card.name, rare_card.id(), ((to_tokens(rare_card.price) * 10) as f64 / discount) as i64));
		let rainbow_card = get_card(ctx, &self.rainbow_card).await;
		desc.push_str(&format!("**5:** {} (_{}_) - {} tokens", rainbow_card.name, rainbow_card.id(), ((to_tokens(rainbow_card.price) * 10) as f64 / discount) as i64));
		ret
			.description(&desc)
			.colour(Colour::from_rgb(255, 50, 20))
			.footer(|f| {
				let local_timer: DateTime<Local> = DateTime::from(self.reset);

				f.text(&format!("Resets {}", local_timer.format("%h %d %H:%M")))
			})
			.author(|a| a
				.icon_url("https://archives.bulbagarden.net/media/upload/9/92/Bag_Coin_Case_Sprite.png")
				.name("Token Shop")
			);

		ret
	}
}

pub fn to_tokens(price: f64) -> i64 {
	let inflation = price * 1.25 * 10.0;

	inflation as i64
}

async fn get_token_shop_collection() -> Collection<TokenShop> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<TokenShop>("tokenshop");

	collection
}

pub async fn get_token_shop(ctx: &Context) -> TokenShop {
	let token_shop_collection = get_token_shop_collection().await;
	let token_shop = token_shop_collection
		.find_one(None, None)
		.await
		.unwrap();
	let token_shop = match token_shop {
		Some(x) => x,
		None => add_token_shop(ctx).await
	};
	if token_shop.reset < Utc::now() {
		let token_shop = token_shop.update_shop(ctx).await;
		update_token_shop(&token_shop).await;
		return token_shop;
	}
	
	token_shop
}

async fn add_token_shop(ctx: &Context) -> TokenShop {
	let ret = TokenShop::new(ctx).await;
	let token_shop_collection = get_token_shop_collection().await;
	token_shop_collection
		.insert_one(&ret, None)
		.await
		.unwrap();
	
	ret
}

async fn update_token_shop(token_shop: &TokenShop) {
	let token_shop_collection = get_token_shop_collection().await;
	token_shop_collection
		.update_one(
			doc! {"_id": &token_shop.id.unwrap() }, 
			doc! {"$set": {
				"sets": &token_shop.sets,
				"rare_card": &token_shop.rare_card,
				"rainbow_card": &token_shop.rainbow_card,
				"reset": &token_shop.reset
			}},
			None)
		.await
		.unwrap();
}

#[command("gamecorner")]
#[aliases("gc", "game", "corner", "gamec")]
#[sub_commands(game_corner_payouts, game_corner_slots, game_corner_tokens_main)]
async fn game_corner_main(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let timer = timers::get_timer().await;
	let mut desc = String::from("Welcome to the **Game Corner**!\n");
	desc.push_str("Here you can play the slot machines to earn tokens that you\n");
	desc.push_str("can convert to cash or spend at the token shop\n\n");
	desc.push_str("Here are the available commands for the Game Corner:\n");
	desc.push_str("**.gamecorner payouts:** Shows the payout information for the slot machines\n");
	desc.push_str("**.gamecorner slots:** Rolls the slot machine\n");
	desc.push_str("**.gamecorner tokenshop:** View the rewards available for purchase\n");
	desc.push_str("**.gamecorner tokenshop buy:** Buys an item from the token shop\n");
	desc.push_str("**.gamecorner tokenshop convert:** Converts your tokens to cash\n\n");
	desc.push_str(&format!("You have **{}** slot rolls remaining", player.daily_slots));
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.description(&desc)
					.colour(Colour::from_rgb(255, 50, 20))
					.footer(|f| {
						let local_timer: DateTime<Local> = DateTime::from(timer.slot_reset);

						f.text(&format!("Resets {}", local_timer.format("%h %d %H:%M")))
					})
					.author(|a| a
						.icon_url("https://archives.bulbagarden.net/media/upload/9/92/Bag_Coin_Case_Sprite.png")
						.name("Game Corner")
					)
			})
		})
		.await?;

	Ok(())
}

#[command("payouts")]
#[aliases("p")]
async fn game_corner_payouts(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let player_slot_mult = 1.0 + player.upgrades.slot_reward_mult as f64 * 0.1;
	let mut desc = String::from("Here are the token payouts for the slot machines\n");
	desc.push_str(&format!("<:GameCorner:967522912242384906><:GameCorner:967522912242384906><:GameCorner:967522912242384906> **{}**\n", (500.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912166903858><:GameCorner:967522912166903858><:GameCorner:967522912166903858> **{}**\n", (200.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912196239510><:GameCorner:967522912196239510><:GameCorner:967522912196239510> **{}**\n", (120.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912275922995><:GameCorner:967522912275922995><:GameCorner:967522912275922995> **{}**\n", (80.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912154296410><:GameCorner:967522912154296410><:GameCorner:967522912154296410> **{}**\n", (50.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912229793882><:GameCorner:967522912229793882><:GameCorner:967522912229793882> **{}**\n", (30.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912166871080><:GameCorner:967522912166871080><:GameCorner:967522912166871080> **{}**\n", (15.0 * player_slot_mult) as i64));
	desc.push_str(&format!("<:GameCorner:967522912166871080><:GameCorner:967522912166871080><:GameCorner:967591653135228988> **{}**", (5.0 * player_slot_mult) as i64));
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.description(&desc)
					.colour(Colour::from_rgb(255, 50, 20))
					.author(|a| a
						.icon_url("https://archives.bulbagarden.net/media/upload/9/92/Bag_Coin_Case_Sprite.png")
						.name("Game Corner")
					)
			})
		})
		.await?;

	Ok(())
}

#[command("slots")]
#[aliases("s")]
async fn game_corner_slots(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = get_player(msg.author.id.0).await;
	if player.daily_slots <= 0 {
		msg.reply(&ctx.http, "You're out of slot rolls for today!").await?;
		return Ok(());
	}
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let amount = if amount < 1 {
		1i64
	} else {
		amount
	};
	let amounts = vec![player.daily_slots, amount]; 
	let amount = *amounts.iter().min().unwrap();
	let slots = Slot::new(amount);
	let mut roll_displays = vec![];
	let mut under_2k_reply = String::from("");
	for roll in slots.rolls {
		let reward = roll.reward(player.upgrades.slot_reward_mult);
		player.tokens += reward;
		player.total_tokens += reward;
		player.slots_rolled += 1;
		player.daily_slots -= 1;
		match (roll.slot1.as_str(), roll.slot2.as_str(), roll.slot3.as_str()) {
			("7", "7", "7") => player.jackpots += 1,
			("7", "7", "R") => player.boofs += 1,
			_ => ()
		}
		let roll_display = roll.reward_display(player.upgrades.slot_reward_mult);
		if roll_display.len() + under_2k_reply.len() >= 1900 { // Much lower than 2000 to account for the varying reward amount and new lines
			roll_displays.push(under_2k_reply);
			under_2k_reply = String::from("");
		}
		under_2k_reply.push_str(&format!("{}\n", roll_display));
	}
	if under_2k_reply.len() > 0 {
		roll_displays.push(under_2k_reply);
	}
	for roll_display in roll_displays {
		msg.reply(&ctx.http, roll_display).await?;
	}
	let mut player_update = Document::new();
	player_update.insert("tokens", player.tokens);
	player_update.insert("total_tokens", player.total_tokens);
	player_update.insert("slots_rolled", player.slots_rolled);
	player_update.insert("daily_slots", player.daily_slots);
	player_update.insert("jackpots", player.jackpots);
	player_update.insert("boofs", player.boofs);
	update_player(&player, doc! { "$set": player_update }).await;

	Ok(())
}

#[command("tokenshop")]
#[aliases("ts", "tokens")]
#[sub_commands(game_corner_tokens_buy, game_corner_tokens_convert)]
async fn game_corner_tokens_main(ctx: &Context, msg: &Message) -> CommandResult {
	let token_shop = get_token_shop(ctx).await;
	let player = get_player(msg.author.id.0).await;
	let embed = token_shop.embed_with_player(ctx, player).await;
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.set_embed(embed);

			m
		})
		.await;

	Ok(())
}

#[command("buy")]
#[aliases("b")]
async fn game_corner_tokens_buy(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let selection = match args.single::<i32>() {
		Ok(x) => x,
		Err(_) => 0
	};
	let token_shop = get_token_shop(ctx).await;
	if !(1..=5).contains(&selection) {
		msg.channel_id.send_message(&ctx.http, |m| m.content("A selection was not made.")).await?;
		return Ok(());
	}
	let amount = match args.single::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let mut update = Document::new();
	let mut player = get_player(msg.author.id.0).await;
	let discount = 1.0 + player.upgrades.tokenshop_discount as f64 * 0.05;
	if selection <= 3 {
		let set = get_set(token_shop.sets.get((selection - 1) as usize).unwrap()).await.unwrap();
		let base_cost = (to_tokens(set.pack_price()) as f64 / discount) as i64;
		if player.tokens < base_cost {
			msg.reply(&ctx.http, &format!("You don't have enough... You need **{}** more tokens", base_cost - player.tokens)).await?;
			return Ok(());
		}
		let total_cost = base_cost * amount;
		let amount = vec![total_cost / base_cost, player.tokens / base_cost]
			.into_iter()
			.min()
			.unwrap(); // Either the most they can afford or the amount they wanted.
		player.tokens -= base_cost * amount;
		*player.packs.entry(set.id()).or_insert(0) += amount;
		player.packs_bought += amount;
		msg.reply(&ctx.http, format!("You bought {} **{}** packs!", amount, set.name)).await?;
		update.insert("tokens", player.tokens);
		update.insert("packs_bought", player.packs_bought);
		let mut player_packs = Document::new();
		for (set_id, amt) in player.packs.iter() {
			player_packs.insert(set_id, amt.clone());
		}
		update.insert("packs", player_packs);
	} else {
		let card = match selection {
			4 => get_card(ctx, &token_shop.rare_card).await,
			_ => get_card(ctx, &token_shop.rainbow_card).await
		};
		let base_cost = ((to_tokens(card.price) * 10) as f64 / discount) as i64;
		if player.tokens < base_cost {
			msg.reply(&ctx.http, &format!("You don't have enough... You need **{}** more tokens", base_cost - player.tokens)).await?;
			return Ok(());
		}
		let total_cost = base_cost * amount;
		let amount = vec![total_cost / base_cost, player.tokens / base_cost]
			.into_iter()
			.min()
			.unwrap(); // Either the most they can afford or the amount they wanted.
		player.tokens -= base_cost * amount;
		*player.cards.entry(card.id()).or_insert(0) += amount;
		msg.reply(&ctx.http, format!("You bought {} **{}**!", amount, card.name)).await?;
		update.insert("tokens", player.tokens);
		let mut player_cards = Document::new();
		for (set_id, amt) in player.cards.iter() {
			player_cards.insert(set_id, amt.clone());
		}
		update.insert("cards", player_cards);
	}
	update_player(&player, doc! { "$set": update }).await;

	Ok(())
}

#[command("convert")]
#[aliases("c")]
async fn game_corner_tokens_convert(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let amount = match args.single::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let mut player = get_player(msg.author.id.0).await;
	if player.tokens <= 0 {
		msg.reply(&ctx.http, "You don't have any tokens").await?;
		return Ok(());
	}
	let mut update = Document::new();
	let amounts = vec![player.tokens, amount];
	let amount = amounts
		.iter()
		.min()
		.unwrap();
	player.tokens -= amount;
	let cash = *amount as f64 * 0.10;
	player.cash += cash;
	player.total_cash += cash;
	msg.reply(&ctx.http, format!("You converted **{}** tokens into **${:.2}**", amount, cash)).await?;
	update.insert("tokens", player.tokens);
	update.insert("cash", player.cash);
	update.insert("total_cash", player.total_cash);
	update_player(&player, doc!{ "$set": update}).await;

	Ok(())
}
