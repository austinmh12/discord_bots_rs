use chrono::{
	Utc,
	Duration,
	DateTime,
	Local,
};
use dotenv;
use mongodb::{
	bson::{
		doc,
		Document
	},
};
use std::{time::Duration as StdDuration, sync::Arc, collections::HashMap};
pub mod card;
use card::SEARCH_CARD_COMMAND;
pub mod sets;
use sets::{SEARCH_SET_COMMAND};
pub mod packs;
pub mod player;
pub mod store;
pub mod player_card;
pub mod timers;
pub mod trade;
use trade::TRADE_WITH_COMMAND;
pub mod slot;
pub mod upgrade;
pub mod quiz;
pub mod binder;
pub mod card_image;
pub mod decks;

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
			ReactionType
		},
	},
	prelude::*
};
use async_trait::async_trait;

use serde_json;
use rand::{
	Rng
};
use crate::{BOTTEST_CHECK, Cache, CardCache};

async fn api_call(endpoint: &str, params: Option<Vec<(&str, &str)>>) -> Option<serde_json::Value> {
	dotenv::dotenv().ok();
	let poketcg_key = dotenv::var("POKETCGAPIKEY").unwrap();
	let client = reqwest::Client::new();

	let mut req = client
		.get(format!("https://api.pokemontcg.io/v2/{}", endpoint))
		.header("X-Api-Key", poketcg_key);
	req = match params {
		Some(x) => req.query(&x),
		None => req
	};
	let data: serde_json::Value = req
		.send().await.unwrap()
		.json().await.unwrap();
	

	if data.is_null() {
		None
	} else {
		Some(data)
	}
}

const RARITY_ORDER: &'static [&str] = &[
	"Rare Secret",
	"Rare Shiny GX",
	"LEGEND",
	"Rare Rainbow",
	"Rare Shiny",
	"Rare Ultra",
	"Rare Holo Star",
	"Rare ACE",
	"Rare BREAK",
	"Rare Holo VMAX",
	"Rare Prime",
	"Rare Prism Star",
	"Rare Holo EX",
	"Rare Holo GX",
	"Rare Holo LV.X",
	"Rare Holo V",
	"Amazing Rare",
	"Rare Shining",
	"Rare Holo",
	"Rare",
	"Promo",
	"Uncommon",
	"Common",
	"Unknown",
];

lazy_static! {
	static ref RARITY_MAPPING: HashMap<&'static str, i64> = {
		let mut m = HashMap::new();
		m.insert("Rare", 75);
		m.insert("Rare ACE", 10);
		m.insert("Rare BREAK", 10);
		m.insert("Rare Holo", 40);
		m.insert("Rare Holo EX", 12);
		m.insert("Rare Holo GX", 12);
		m.insert("Rare Holo LV.X", 12);
		m.insert("Rare Holo Star", 8);
		m.insert("Rare Holo V", 15);
		m.insert("Rare Holo VMAX", 10);
		m.insert("Rare Prime", 10);
		m.insert("Rare Prism Star", 10);
		m.insert("Rare Rainbow", 5);
		m.insert("Rare Secret", 1);
		m.insert("Rare Shining", 20);
		m.insert("Rare Shiny", 5);
		m.insert("Rare Shiny GX", 2);
		m.insert("Rare Ultra", 5);
		m.insert("Amazing Rare", 15);
		m.insert("LEGEND", 3);
		
		m
	};
}

pub trait PaginateEmbed {
	fn embed(&self) -> CreateEmbed;
}

pub trait CardInfo {
	fn card_id(&self) -> String;
	fn card_name(&self) -> String;
	fn description(&self) -> String;
	fn price(&self) -> f64;
}

pub trait Idable {
	fn id(&self) -> String;
}

pub trait HasSet {
	fn set(&self) -> sets::Set;
}

#[async_trait]
pub trait Scrollable {
	async fn scroll_through(&self, ctx: &Context, msg: &Message) -> Result<(), String>;
}

enum SellMode {
	Under(f64),
	Duplicates,
	All,
	BySet(String),
}

async fn binder_paginated_embeds(ctx: &Context, msg: &Message, player: player::Player, missing_only: bool) -> Result<(), String> {
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let set = sets::get_set(&player.current_binder.set).await.unwrap();
	let mut set_cards = card::get_cards_by_set(ctx, &set).await;
	set_cards.sort_by(|c1, c2| {
		if c1.set().id() == c2.set().id() {
			let c1_num = c1.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);
			let c2_num = c2.id().split("-").collect::<Vec<&str>>()[1].parse::<i64>().unwrap_or(999);

			c1_num.cmp(&c2_num)
		} else {
			c1.id().cmp(&c2.id())
		}
	});
	let footer_extra = format!("({}/{} - {:.1}%)", player.current_binder.cards.len(), set_cards.len(), (player.current_binder.cards.len() as f64 / set_cards.len() as f64) * 100.0);
	let cards = if missing_only {
		set_cards
			.iter()
			.filter(|c| !player.current_binder.cards.contains(&c.card_id()))
			.map(|c| c.to_owned())
			.collect::<Vec<card::Card>>()
	} else {
		set_cards
	};
	let embeds = cards.iter().map(|e| e.embed()).collect::<Vec<_>>();
	let mut idx: i16 = 0;
	if !player.current_binder.cards.contains(&cards[idx as usize].card_id()) {
		let card_img = card_image::get_card_image(&cards[idx as usize]).await;
		let img = card_img.to_dyn_image();
		img.save("gs.png").unwrap();
	}
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{} {}", idx + 1, embeds.len(), footer_extra)));
			}
			if !player.current_binder.cards.contains(&cards[idx as usize].card_id()) {
				cur_embed.image("attachment://gs.png");
				m.add_file("gs.png");
			}
			m.set_embed(cur_embed);

			if embeds.len() > 1 {
				m.reactions([left_arrow.clone(), right_arrow.clone()]);
			}

			m			
		}).await.unwrap();
	loop {
		if embeds.len() <= 1 {
			break; // Exit before anything. Probably a way to do this before entering.
		}
		if let Some(reaction) = &message
			.await_reaction(&ctx)
			.timeout(StdDuration::from_secs(90))
			.author_id(msg.author.id)
			.removed(true)
			.await
		{
			let emoji = &reaction.as_inner_ref().emoji;
			match emoji.as_data().as_str() {
				"⬅️" => idx = (idx - 1).rem_euclid(embeds.len() as i16),
				"➡️" => idx = (idx + 1) % embeds.len() as i16,
				_ => continue
			};
		} else {
			message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
			break;
		}
		let in_binder = player.current_binder.cards.contains(&cards[idx as usize].card_id());
		if !in_binder {
			let card_img = card_image::get_card_image(&cards[idx as usize]).await;
			let img = card_img.to_dyn_image();
			img.save("gs.png").unwrap();
		}
		message.edit(&ctx, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{} {}", idx + 1, embeds.len(), footer_extra)));
			}
			if !in_binder {
				cur_embed.image("attachment://gs.png");
				m.attachment("gs.png");
			}
			m.set_embed(cur_embed);

			m
		}).await.unwrap();

		// This needs to be done after setting the embed without the attachment
		// otherwise the attachment won't be removed until the next cycle of a card in
		// the binder.
		if in_binder {
			let attachments = message.attachments.clone();
			message.edit(&ctx, |m| {
				for attachment in attachments {
					m.remove_existing_attachment(attachment.id);
				}

				m
			}).await.unwrap();
		}
	}

	Ok(())
}

async fn get_set_average_price(ctx: &Context, set: &sets::Set) -> f64 {
	let all_cards = card::get_cards_by_set(ctx, set).await;
	let rares = all_cards
		.iter()
		.filter(|c| c.rarity != "Common" || c.rarity != "Uncommon" || c.rarity != "Promo")
		.map(|c| c.to_owned())
		.collect::<Vec<card::Card>>();
	let uncommons = all_cards
		.iter()
		.filter(|c| c.rarity == "Uncommon")
		.map(|c| c.to_owned())
		.collect::<Vec<card::Card>>();
	let commons = all_cards
		.iter()
		.filter(|c| c.rarity == "Common")
		.map(|c| c.to_owned())
		.collect::<Vec<card::Card>>();
	let promos = all_cards
		.iter()
		.filter(|c| c.rarity == "Promo" || c.rarity == "Classic Collection" || c.rarity == "Unknown")
		.map(|c| c.to_owned())
		.collect::<Vec<card::Card>>();
	if commons.len() > 0 && uncommons.len() > 0 && rares.len() > 0 {
		let common_price: f64 = (commons.iter().map(|c| c.price).sum::<f64>() / commons.len() as f64) * 6.0;
		let uncommon_price: f64 = (uncommons.iter().map(|c| c.price).sum::<f64>() / uncommons.len() as f64) * 3.0;
		let mut rare_prices = vec![];
		for (rarity, weight) in RARITY_MAPPING.iter() {
			let current_rarity_cards = rares
				.iter()
				.filter(|c| &c.rarity.as_str() == rarity)
				.map(|c| c.to_owned())
				.collect::<Vec<card::Card>>();
			if current_rarity_cards.len() == 0 {
				continue;
			}
			let rarity_price: f64 = current_rarity_cards.iter().map(|c| c.price).sum::<f64>() / current_rarity_cards.len() as f64;
			rare_prices.push((rarity_price, weight));
		}
		let rares_price: f64 = rare_prices.iter().map(|rp| rp.0 * *rp.1 as f64).sum::<f64>() / rare_prices.iter().map(|rp| *rp.1 as f64).sum::<f64>();
		return common_price + uncommon_price + rares_price;
	} else {
		let promo_price: f64 = promos.iter().map(|c| c.price).sum::<f64>() / promos.len() as f64;
		return promo_price;
	}
}

#[command("sell")]
#[sub_commands(sell_card, sell_under, sell_dups, sell_all, sell_packs, sell_set)]
async fn sell_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available selling commands:
	**.sell card <card id> [amount - Default: _1_]** to sell a specific card.
	**.sell under [value - Default: _1.00_] [rares - Default: _false_]** to sell all cards worth less than the value entered.
	**.sell dups [rares - Default: _false_]** to sell all duplicate cards until 1 remains. Doesn\'t sell rares by default.
	**.sell all [rares - Default: _false_]** to sell all cards. Doesn\'t sell rares by default.
	**.sell set <set id> [rares - Default: _false_]** to sell all cards from a specific set. Doesn\'t sell rares by default.
	**.sell packs <set id> [amount - Default: 1]** to sell a pack.";
	msg
		.channel_id
		.send_message(&ctx.http, |m| m.content(content))
		.await?;

	Ok(())
}

async fn sell_cards_helper(ctx: &Context, mut player: player::Player, mode: SellMode, rares: bool) -> (Vec<player_card::PlayerCard>, i64, f64, Document) {
	// Does the actual removal and calculation of the cards worths
	// Returns the list of cards sold, total sold, total earned, and the document to update the player with
	let player_cards = player_card::player_cards(ctx, player.cards.clone()).await;
	let mut cards_to_sell = vec![];
	for player_card in player_cards {
		let sellable = match mode {
			SellMode::Under(value) => player_card.price() <= value,
			SellMode::Duplicates => player_card.amount > 1,
			SellMode::All => true,
			SellMode::BySet(ref set_id) => player_card.set().id().as_str() == set_id
		};
		// If it fails the first filter, do nothing
		if !sellable {
			continue;
		}
		let sellable = if rares {
			true
		} else if vec!["Common", "Uncommon"].contains(&player_card.card.rarity.as_str()) {
			true
		} else {
			false
		};
		// If it fails the second filter, do nothing
		if !sellable {
			continue;
		}
		// If all the sellable filters pass, add the card with an amount based on the savelist
		match player.savelist.contains(&player_card.card_id()) {
			true => cards_to_sell.push((player_card.clone(), player_card.amount - 1)),
			false => cards_to_sell.push((player_card.clone(), player_card.amount))
		}
	}
	let mut total_sold = 0;
	let mut total_cash = 0.00;
	let mut sold_cards = vec![];
	for (card_to_sell, amount) in cards_to_sell.clone() {
		*player.cards.entry(card_to_sell.card_id()).or_insert(0) -= amount;
		total_sold += amount;
		total_cash += match player.completed_binders.contains(&card_to_sell.set().id()) {
			true => amount as f64 * card_to_sell.price() * 1.25,
			false => amount as f64 * card_to_sell.price()
		};
		sold_cards.push(card_to_sell.clone());
	}
	player.cards.retain(|_, v| *v > 0);
	player.cash += total_cash;
	player.total_cash += total_cash;
	player.cards_sold += total_sold;
	let mut player_update = Document::new();
	let mut player_card_update = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_card_update.insert(crd, amt);
	}
	player_update.insert("cards", player_card_update);
	player_update.insert("cards_sold", player.cards_sold);
	player_update.insert("cash", player.cash);
	player_update.insert("total_cash", player.total_cash);

	(sold_cards, total_sold, total_cash, player_update)
}

#[command("card")]
#[aliases("c")]
async fn sell_card(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let card_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if card_id == "" {
		msg.reply(&ctx.http, "No card provided").await?;
		return Ok(());
	}
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	
	let mut player = player::get_player(msg.author.id.0).await;
	if player.cards.contains_key(&card_id) {
		let amounts = vec![amount, *player.cards.get(&card_id).unwrap()];
		let amount = *amounts.iter().min().unwrap();
		let card = card::get_card(ctx, &card_id).await;
		let mut update = Document::new();
		*player.cards.entry(card_id.clone()).or_insert(0) -= amount;
		if *player.cards.entry(card_id.clone()).or_insert(0) == 0 {
			player.cards.remove(&card_id);
		}
		player.cards_sold += amount;
		player.cash += card.price * amount as f64;
		player.total_cash += card.price * amount as f64;
		update.insert("cards_sold", player.cards_sold);
		update.insert("cash", player.cash);
		update.insert("total_cash", player.total_cash);
		let mut player_cards = Document::new();
		for (crd, amt) in player.cards.iter() {
			player_cards.insert(crd, amt);
		}
		update.insert("cards", player_cards);
		player::update_player(&player, doc! { "$set": update }).await;
		msg.reply(&ctx.http, format!("You sold {} **{}** for ${:.2}", amount, card.name, card.price * amount as f64)).await?;
	} else {
		msg.reply(&ctx.http, "You don't have that card").await?;
	}

	Ok(())
}

#[command("under")]
#[aliases("u")]
async fn sell_under(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let value = match args.find::<f64>() {
		Ok(x) => x,
		Err(_) => 1.00
	};
	let rares = match args.find::<bool>() {
		Ok(x) => x,
		Err(_) => false
	};
	let player = player::get_player(msg.author.id.0).await;
	let (_, total_sold, total_cash, player_update) = sell_cards_helper(ctx, player.clone(), SellMode::Under(value), rares).await;
	player::update_player(&player, doc! { "$set": player_update }).await;
	msg.reply(&ctx.http, format!("You sold **{}** cards for **${:.2}**", total_sold, total_cash)).await?;

	Ok(())
}

#[command("dups")]
#[aliases("dup", "d")]
async fn sell_dups(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let rares = match args.find::<bool>() {
		Ok(x) => x,
		Err(_) => false
	};
	let player = player::get_player(msg.author.id.0).await;
	let (_, total_sold, total_cash, player_update) = sell_cards_helper(ctx, player.clone(), SellMode::Duplicates, rares).await;
	player::update_player(&player, doc! { "$set": player_update }).await;
	msg.reply(&ctx.http, format!("You sold **{}** cards for **${:.2}**", total_sold, total_cash)).await?;

	Ok(())
}

#[command("all")]
#[aliases("a")]
async fn sell_all(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let rares = match args.find::<bool>() {
		Ok(x) => x,
		Err(_) => false
	};
	let player = player::get_player(msg.author.id.0).await;
	let (_, total_sold, total_cash, player_update) = sell_cards_helper(ctx, player.clone(), SellMode::All, rares).await;
	player::update_player(&player, doc! { "$set": player_update }).await;
	msg.reply(&ctx.http, format!("You sold **{}** cards for **${:.2}**", total_sold, total_cash)).await?;

	Ok(())
}

#[command("set")]
#[aliases("s")]
async fn sell_set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let set_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	let set = sets::get_set(&set_id).await;
	match set {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "No set found with that id.").await?;
		}
	}
	let set = set.unwrap();
	let rares = match args.find::<bool>() {
		Ok(x) => x,
		Err(_) => false
	};
	let player = player::get_player(msg.author.id.0).await;
	let (_, total_sold, total_cash, player_update) = sell_cards_helper(ctx, player.clone(), SellMode::BySet(set.id()), rares).await;
	player::update_player(&player, doc! { "$set": player_update }).await;
	msg.reply(&ctx.http, format!("You sold **{}** cards for **${:.2}**", total_sold, total_cash)).await?;

	Ok(())
}

#[command("packs")]
#[aliases("pack", "p")]
async fn sell_packs(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let pack_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if pack_id == "" {
		msg.reply(&ctx.http, "No pack provided").await?;
		return Ok(());
	}
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	
	let mut player = player::get_player(msg.author.id.0).await;
	if player.packs.contains_key(&pack_id) {
		let amounts = vec![amount, *player.packs.get(&pack_id).unwrap()];
		let amount = *amounts.iter().min().unwrap();
		let set = sets::get_set(&pack_id).await.unwrap();
		let mut update = Document::new();
		*player.packs.entry(pack_id.clone()).or_insert(0) -= amount;
		player.packs.retain(|_, v| *v > 0);
		player.cash += set.pack_price() * amount as f64;
		update.insert("cash", player.cash);
		let mut player_packs = Document::new();
		for (pck, amt) in player.packs.iter() {
			player_packs.insert(pck, amt);
		}
		update.insert("packs", player_packs);
		player::update_player(&player, doc! { "$set": update }).await;
		msg.reply(&ctx.http, format!("You sold {} **{}** packs for ${:.2}", amount, set.name, set.pack_price() * amount as f64)).await?;
	} else {
		msg.reply(&ctx.http, "You don't have that card").await?;
	}

	Ok(())
}

#[command("search")]
#[sub_commands(search_card, search_set)]
async fn search_main(ctx: &Context, msg: &Message) -> CommandResult {
	let search_help_str = "Here are the available **search** commands:
	**.search card:** Searches for a card with a matching name
	**.search set:** Searches for a set with a matching name

	**Basic searching:** https://pokemontcg.guru/
	**Advanced searching:** https://pokemontcg.guru/advanced
	**Searching syntax:** http://www.lucenetutorial.com/lucene-query-syntax.html";
	msg.reply(&ctx.http, search_help_str).await?;

	Ok(())
}

#[command("openpack")]
#[aliases("op")]
async fn open_pack_command(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	if player.daily_packs <= 0 {
		msg.reply(&ctx.http, "You're out of packs for today!").await?;
		return Ok(());
	}
	let set_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if set_id == "" {
		msg.reply(&ctx.http, "No pack provided.").await?;
		return Ok(());
	}
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	if player.packs.contains_key(&set_id) {
		let amounts = vec![player.daily_packs, amount, *player.packs.get(&set_id).unwrap()]; 
		let amount = *amounts.iter().min().unwrap();
		let pack = packs::Pack::from_set_id(ctx, &set_id, amount as usize).await?;
		let mut update = Document::new();
		player.total_cards += pack.cards.len() as i64;
		player.packs_opened += amount;
		player.daily_packs -= amount;
		*player.packs.entry(set_id.clone()).or_insert(0) -= amount;
		if *player.packs.entry(set_id.clone()).or_insert(0) == 0 {
			player.packs.remove(&set_id);
		}
		update.insert("total_cards", player.total_cards);
		update.insert("packs_opened", player.packs_opened);
		update.insert("daily_packs", player.daily_packs);
		let mut player_packs = Document::new();
		for (set, amt) in player.packs.iter() {
			player_packs.insert(set, amt);
		}
		update.insert("packs", player_packs);
		let mut player_cards = Document::new();
		for card in &pack.cards {
			*player.cards.entry(card.id()).or_insert(0) += 1;
		}
		for (card_id, amt) in player.cards.iter() {
			player_cards.insert(card_id, amt);
		}
		update.insert("cards", player_cards);
		player::update_player(&player, doc! { "$set": update }).await;
		pack.cards.scroll_through(ctx, msg).await?;
	} else {
		msg.reply(&ctx.http, "You don't have that pack").await?;
	}

	Ok(())
}

#[command("daily")]
#[aliases("d")]
async fn daily_command(ctx: &Context, msg: &Message) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	let now = Utc::now();
	if player.daily_reset >= now {
		msg
			.channel_id
			.send_message(
				&ctx.http, |m| {
					let local_timer: DateTime<Local> = DateTime::from(player.daily_reset);
				
					m.content(format!("Your daily resets **{}**", local_timer.format("%h %d %H:%M")))
				})
			.await
			.unwrap();
		return Ok(());
	}
	let mut update = Document::new();
	let r: i64 = rand::thread_rng().gen_range(0..100);
	if r <= 2 {
		let player_daily_packs = 50 + (player.upgrades.daily_pack_amount * 10);
		player.daily_packs += player_daily_packs;
		update.insert("daily_packs", player.daily_packs);
		msg.reply(&ctx.http, "***WOAH!*** Your daily packs were reset!").await?;
	} else {
		let player_mult = 1.0 + player.upgrades.daily_reward_mult as f64 * 0.1;
		let cash: f64 = rand::thread_rng().gen_range(5..=20) as f64 * player_mult;
		player.cash += cash;
		player.total_cash += cash;
		update.insert("cash", player.cash);
		update.insert("total_cash", player.total_cash);
		msg.reply(&ctx.http, format!("You got **${:.2}**", cash as f64)).await?;
	}
	let hours_til_update = 24 - player.upgrades.daily_time_reset;
	player.daily_reset = Utc::now() + Duration::hours(hours_til_update);
	update.insert("daily_reset", player.daily_reset);
	player::update_player(&player, doc!{ "$set": update }).await;

	Ok(())
}

#[command("trade")]
#[aliases("tr")]
#[sub_commands(trade_with)]
async fn trade_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available trading commands:
		**.trade with <@player> <trade offer>** to trade with another player

		The **trade offer** is written as **cardID:amount/packID:amount/$cashamount**
		E.g. to trade a **Jigglypuff** for a **Magikarp** player 1 would use:
		**.trade with @player2 bwp-bw65**, player 2 would reply **xyp-xy143**
		Trading multiple would make the trade offer: **bwp-bw65/dp2-108:2**
		Which would offer a Jigglypuff and two Zubats.

		Here are some trading examples in **offer** | **response**:
			**.trade with @player2 bwp:2/$10** | **dp2-108:5**
				Offers 2 bwp packs and $10 for 5 Zubats
			**.trade with @player2 $25** | **xyp-xy143/xyp**
				Offers $25 for a Magikarp and an xyp pack";
	msg
		.channel_id
		.send_message(&ctx.http, |m| m.content(content))
		.await?;

	Ok(())
}

// ADMIN COMMANDS (FOR TESTING)
#[command("admin")]
#[sub_commands(admin_show_pack, admin_add_cash, admin_mock_slot, admin_add_tokens, admin_set_cards, admin_cache)]
#[checks(BotTest)]
async fn admin_main() -> CommandResult {
	Ok(())
}

#[command("pack")]
#[checks(BotTest)]
async fn admin_show_pack(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	let set_id = args.find::<String>().unwrap();
	let amount = match args.find::<i32>() {
		Ok(x) => x as usize,
		Err(_) => 1usize
	};
	let pack = packs::Pack::from_set_id(ctx, set_id.as_str(), amount).await.unwrap();
	pack.cards.scroll_through(ctx, msg).await?;

	Ok(())
}

#[command("cash")]
#[checks(BotTest)]
async fn admin_add_cash(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player_ = player::get_player(msg.author.id.0).await;
	let amount = args.find::<f64>().expect("No amount to add");
	let og_cash = player_.cash;
	player_.cash += amount;
	player_.total_cash += amount;
	msg.reply(&ctx.http, format!("{} had **${:.2}**, now they have **${:.2}**", &player_.discord_id, og_cash, &player_.cash))
		.await?;
	player::update_player(
		&player_,
		doc! {
			"$set": { 
				"cash": &player_.cash,
				"total_cash": &player_.total_cash
			}
		}
	)
		.await;

	Ok(())
}

#[command("tokens")]
#[checks(BotTest)]
async fn admin_add_tokens(_ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	let amount = args.find::<i64>().expect("No amount to add");
	player.tokens += amount;
	player.total_tokens += amount;
	player::update_player(
		&player,
		doc! {
			"$set": { 
				"tokens": &player.tokens,
				"total_tokens": &player.total_tokens
			}
		}
	)
		.await;

	Ok(())
}

#[command("slots")]
#[checks(BotTest)]
async fn admin_mock_slot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	if player.daily_slots <= 0 {
		msg.reply(&ctx.http, "You're out of slot rolls for today!").await?;
		return Ok(());
	}
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let slots = slot::Slot::new(amount);
	let mut roll_displays = vec![];
	for roll in slots.rolls {
		roll_displays.push(roll.reward_display(player.upgrades.slot_reward_mult));
	}
	let content = roll_displays.join("\n");

	msg.reply(&ctx.http, content).await?;

	Ok(())
}

#[command("set")]
#[checks(BotTest)]
async fn admin_set_cards(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	let set_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	let set = sets::get_set(&set_id).await;
	match set {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "No set found with that id.").await?;
		}
	}
	let set = set.unwrap();
	let cards = card::get_cards_by_set(ctx, &set).await;
	for card in cards {
		*player.cards.entry(card.card_id()).or_insert(0) += 1;
		if *player.cards.entry(card.card_id()).or_insert(0) == 0 {
			player.cards.remove(&card.card_id());
		}
	}
	let mut player_update = Document::new();
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	player_update.insert("cards", player_cards);
	player::update_player(&player, doc! { "$set": player_update }).await;
	msg.reply(&ctx.http, format!("Added all the cards for **{}**", set.name)).await?;

	Ok(())
}

#[command("cache")]
#[checks(BotTest)]
async fn admin_cache(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	{
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		println!("{:?}", cache.iter());

	}

	Ok(())
}

// TASKS
pub async fn refresh_dailys(_ctx: Arc<Context>) {
	let timer = timers::get_timer().await;
	if Utc::now() >= timer.pack_reset {
		println!("Reseting dailys");
		let players = player::get_players().await;
		for mut player in players {
			let mut update = Document::new();
			let player_daily_packs = 50 + (player.upgrades.daily_pack_amount * 10);
			let player_daily_slots = 10 + player.upgrades.daily_slot_amount;
			if player.daily_packs < player_daily_packs {
				player.daily_packs = player_daily_packs;
				update.insert("daily_packs", player.daily_packs);
			}
			player.daily_slots = player_daily_slots;
			update.insert("daily_slots", player.daily_slots);
			player::update_player(&player, doc!{"$set": update }).await;
		}
		timers::update_timer(&timer).await;
	}
}

pub async fn refresh_card_prices(ctx: Arc<Context>) {
	let cached_cards = card::get_outdated_cards(&ctx).await;
	print!("Updating {} outdated cards... ", &cached_cards.len());
	let card_ids = cached_cards
		.iter()
		.map(|c| c.card.id())
		.collect::<Vec<String>>();
	let refreshed_cards = card::get_multiple_cards_by_id_without_cache(card_ids).await;
	let mut updated_cards: Vec<CardCache> = vec![];
	for mut cached_card in cached_cards {
		let refreshed_card = refreshed_cards.get(&cached_card.card.id()).unwrap();
		cached_card.card.price = refreshed_card.price;
		cached_card.last_updated = Utc::now() + Duration::days(1);
		updated_cards.push(cached_card);
	}
	card::update_cached_cards(&ctx, updated_cards).await;
	println!("Updated cached cards!");
}

/* Tasks
 * v1.6.0
 * 	Add global cash sinks (buys for all players) (aka expensive)
 * 		daily reset
 * 		token shop refresh
 * 		store refresh
 * 		pack reset
 * 		slot reset
 * Misc
 * 	Add a help command (Okay shits not working so I don't care right now.)
 *	 	Add usage macro to all the commands and sub commands
 * 	Add a changelog command that DMs the user the patch note they want
 * 		Add the current version to the activity
*/