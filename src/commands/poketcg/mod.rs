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
use std::{time::Duration as StdDuration, sync::Arc, cmp::Ordering};
pub mod card;
pub mod sets;
use sets::get_set;
pub mod packs;
pub mod player;
pub mod store;
pub mod player_card;
pub mod timers;
pub mod trade;
pub mod slot;
use player_card::{
	player_cards
};

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
	utils::{
		Colour
	},
	prelude::*
};

use serde_json;
use rand::{
	Rng
};
use crate::BOTTEST_CHECK;

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

pub trait PaginateEmbed {
	fn embed(&self) -> CreateEmbed;
}

pub trait CardInfo {
	fn card_id(&self) -> String;
	fn card_name(&self) -> String;
}

pub trait Idable {
	fn id(&self) -> String;
}

// paginated embeds to search through cards
async fn paginated_embeds<T:PaginateEmbed>(ctx: &Context, msg: &Message, embeds: Vec<T>) -> Result<(), String> {
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let embeds = embeds.iter().map(|e| e.embed()).collect::<Vec<_>>();
	let mut idx: i16 = 0;
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
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
		message.edit(&ctx, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			m
		}).await.unwrap();
	}

	Ok(())
}

async fn card_paginated_embeds<T:CardInfo + PaginateEmbed>(ctx: &Context, msg: &Message, cards: Vec<T>, mut player: player::Player) -> Result<(), String> {
	// TODO: Find a way to tell if something is in your savelist
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let save_icon = ReactionType::try_from("💾").expect("No floppy disk");
	let embeds = cards.iter().map(|e| e.embed()).collect::<Vec<_>>();
	let mut idx: i16 = 0;
	let mut content = String::from("");
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			if embeds.len() > 1 {
				m.reactions([left_arrow.clone(), right_arrow.clone(), save_icon.clone()]);
			} else {
				m.reactions([save_icon.clone()]);
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
				"💾" => {
					let card_id = &cards[idx as usize].card_id();
					if player.savelist.clone().contains(&card_id) {
						let index = player.savelist.clone().iter().position(|c| c == card_id).unwrap();
						player.savelist.remove(index);
						content = format!("**{}** removed from your savelist!", &cards[idx as usize].card_name());
					} else {
						player.savelist.push(card_id.clone());
						content = format!("**{}** added to your savelist!", &cards[idx as usize].card_name());
					}
					player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;
				}
				_ => continue
			};
		} else {
			message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
			break;
		}
		message.edit(&ctx, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);
			m.content(content);

			m
		}).await.unwrap();

		content = String::from("");
	}

	Ok(())
}

async fn set_paginated_embeds(ctx: &Context, msg: &Message, embeds: Vec<sets::Set>) -> Result<(), String> {
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let pokemon_card = ReactionType::try_from("<:poketcg:965802882433703936>").expect("No TCG Back");
	let player = player::get_player(msg.author.id.0).await;
	let sets = &embeds.clone();
	let embeds = embeds.iter().map(|e| e.embed()).collect::<Vec<_>>();
	let mut idx: i16 = 0;
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			if embeds.len() > 1 {
				m.reactions([left_arrow.clone(), right_arrow.clone(), pokemon_card.clone()]);
			} else {
				m.reactions([pokemon_card.clone()]);
			}

			m			
		}).await.unwrap();
	
	loop {
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
				"poketcg:965802882433703936" => {
					let set = sets.into_iter().nth(idx as usize).unwrap();
					let cards = card::get_cards_by_set(set).await;
					message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
					
					card_paginated_embeds(ctx, msg, cards, player.clone()).await?
				},
				_ => continue
			};
		} else {
			message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
			break;
		}
		message.edit(&ctx, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			m
		}).await.unwrap();
	}

	Ok(())
}

#[command("my")]
#[sub_commands(my_cards, my_packs, my_stats)]
async fn my_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available my commands:
	**.my cards [sort_by - Default: name]** to view your cards.
	**.my packs** to view your packs.
	**.my stats** to view your stats";
	let _ = player::get_player(msg.author.id.0).await;
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
	let player = player::get_player(msg.author.id.0).await;
	let mut cards = player_cards(player.cards.clone()).await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "You have no cards!").await?;
	} else {
		match sorting.replace("-", "").as_str() {
			"id" => cards.sort_by(|c1, c2| c1.card.id().cmp(&c2.card.id())),
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
	let player = player::get_player(msg.author.id.0).await;
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

						f.text(&format!("Resets {}", local_timer.format("%h %d %H:%m")))
					})
			})
		})
		.await?;

	Ok(())
}

#[command("stats")]
async fn my_stats(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
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

#[command("sell")]
#[sub_commands(sell_card, sell_under, sell_dups, sell_all, sell_packs)]
async fn sell_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available selling commands:
	**.sell card <card id> [amount - Default: _1_]** to sell a specific card.
	**.sell under [value - Default: _1.00_] [rares - Default: _false_]** to sell all cards worth less than the value entered.
	**.sell dups [rares - Default: _false_]** to sell all duplicate cards until 1 remains. Doesn\'t sell rares by default.
	**.sell all [rares - Default: _false_]** to sell all cards. Doesn\'t sell rares by default.
	**.sell packs <set id> [amount - Default: 1]** to sell a pack.";
	msg
		.channel_id
		.send_message(&ctx.http, |m| m.content(content))
		.await?;

	Ok(())
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
		let card = card::get_card(&card_id).await;
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
	let mut player = player::get_player(msg.author.id.0).await;
	let mut cards_to_sell = vec![];
	for player_card in player_cards(player.cards.clone()).await {
		if player.savelist.contains(&player_card.card.id()) {
			continue;
		}
		if player_card.card.price <= value {
			if rares {
				cards_to_sell.push(player_card)
			} else if vec!["Common", "Uncommon"].contains(&player_card.card.rarity.as_str()) {
				cards_to_sell.push(player_card)
			} else {
				continue;
			}
		}
	}
	let mut total_sold = 0;
	let mut total_cash = 0.00;
	for card_to_sell in cards_to_sell {
		*player.cards.entry(card_to_sell.card.id()).or_insert(0) -= card_to_sell.amount;
		total_sold += card_to_sell.amount;
		total_cash += card_to_sell.amount as f64 * card_to_sell.card.price;
	}
	player.cards.retain(|_, v| *v > 0);
	let mut update = Document::new();
	player.cards_sold += total_sold;
	player.cash += total_cash;
	player.total_cash += total_cash;
	update.insert("cards_sold", player.cards_sold);
	update.insert("cash", player.cash);
	update.insert("total_cash", player.total_cash);
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	update.insert("cards", player_cards);
	player::update_player(&player, doc! { "$set": update }).await;
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
	let mut player = player::get_player(msg.author.id.0).await;
	let mut cards_to_sell = vec![];
	for player_card in player_cards(player.cards.clone()).await {
		if player.savelist.contains(&player_card.card.id()) {
			continue;
		}
		if player_card.amount > 1 {
			if rares {
				cards_to_sell.push(player_card)
			} else if vec!["Common", "Uncommon"].contains(&player_card.card.rarity.as_str()) {
				cards_to_sell.push(player_card)
			} else {
				continue;
			}
		}
	}
	let mut total_sold = 0;
	let mut total_cash = 0.00;
	for card_to_sell in cards_to_sell {
		let amt = card_to_sell.amount - 1;
		*player.cards.entry(card_to_sell.card.id()).or_insert(0) -= amt;
		total_sold += amt;
		total_cash += amt as f64 * card_to_sell.card.price;
	}
	player.cards.retain(|_, v| *v > 0);
	let mut update = Document::new();
	player.cards_sold += total_sold;
	player.cash += total_cash;
	player.total_cash += total_cash;
	update.insert("cards_sold", player.cards_sold);
	update.insert("cash", player.cash);
	update.insert("total_cash", player.total_cash);
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	update.insert("cards", player_cards);
	player::update_player(&player, doc! { "$set": update }).await;
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
	let mut player = player::get_player(msg.author.id.0).await;
	let mut cards_to_sell = vec![];
	for player_card in player_cards(player.cards.clone()).await {
		if player.savelist.contains(&player_card.card.id()) {
			continue;
		}
		if rares {
			cards_to_sell.push(player_card)
		} else if vec!["Common", "Uncommon"].contains(&player_card.card.rarity.as_str()) {
			cards_to_sell.push(player_card)
		} else {
			continue;
		}
	}
	let mut total_sold = 0;
	let mut total_cash = 0.00;
	for card_to_sell in cards_to_sell {
		*player.cards.entry(card_to_sell.card.id()).or_insert(0) -= card_to_sell.amount;
		total_sold += card_to_sell.amount;
		total_cash += card_to_sell.amount as f64 * card_to_sell.card.price;
	}
	player.cards.retain(|_, v| *v > 0);
	let mut update = Document::new();
	player.cards_sold += total_sold;
	player.cash += total_cash;
	player.total_cash += total_cash;
	update.insert("cards_sold", player.cards_sold);
	update.insert("cash", player.cash);
	update.insert("total_cash", player.total_cash);
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	update.insert("cards", player_cards);
	player::update_player(&player, doc! { "$set": update }).await;
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

#[command("card")]
async fn search_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	let search_str = args.rest();
	let cards = card::get_cards_with_query(&format!("{}", search_str))
		.await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "No cards found.").await?;
	} else {
		card_paginated_embeds(ctx, msg, cards, player).await?;
	}

	Ok(())
}
// NtS: Maybe make .search card name <cardName> and .search card id <cardId> ?
// NtS: (N)ote (t)o (S)elf

#[command("set")]
async fn search_set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let search_str = args.rest();
	let sets = sets::get_sets_with_query(&format!("{}", search_str))
		.await;
	if sets.len() == 0 {
		msg.reply(&ctx.http, "No sets found.").await?;
	} else {
		set_paginated_embeds(ctx, msg, sets).await?;
	}

	Ok(())
}

#[command("sets")]
async fn sets_command(ctx: &Context, msg: &Message) -> CommandResult {
	let sets = sets::get_sets().await;
	set_paginated_embeds(ctx, msg, sets).await?;

	Ok(())
}

#[command("set")]
async fn set_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let set_id = args.rest();
	let set = sets::get_set(set_id).await;
	match set {
		Some(x) => set_paginated_embeds(ctx, msg, vec![x]).await?,
		None => {
			msg.reply(&ctx.http, "No set found with that id.").await?;
		}
	}

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
		let pack = packs::Pack::from_set_id(&set_id, amount as usize).await?;
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
		card_paginated_embeds(ctx, msg, pack.cards, player).await?;
	} else {
		msg.reply(&ctx.http, "You don't have that pack").await?;
	}

	Ok(())
}

#[command("store")]
#[aliases("st")]
#[sub_commands(store_buy)]
async fn store_main(ctx: &Context, msg: &Message) -> CommandResult {
	let store_ = store::get_store().await;
	let player_ = player::get_player(msg.author.id.0).await;
	let embed = store_.embed_with_player(player_).await;
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
async fn store_buy(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut selection = match args.single::<i32>() {
		Ok(x) => x,
		Err(_) => 0
	};
	let selection_str = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	let store_ = store::get_store().await;
	if selection_str != "" && selection == 0 {
		selection = (store_.sets.iter().position(|r| r == &selection_str).unwrap_or(10) + 1) as i32;
	}
	if !(1..=10).contains(&selection) {
		msg.channel_id.send_message(&ctx.http, |m| m.content("A selection was not made.")).await?;
		return Ok(());
	}
	let amount = match args.find::<i32>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let set = get_set(store_.sets.get((selection - 1) as usize).unwrap()).await.unwrap();
	let mut player = player::get_player(msg.author.id.0).await;
	let (price_mult, pack_count) = if selection <= 4 {
		(1.0, 1)
	} else if 5 <= selection && selection <= 7 {
		(2.5, 4)
	} else if 8 <= selection && selection <= 9 {
		(10.0, 12)
	} else {
		(30.0, 36)
	};
	let base_cost = set.pack_price() * price_mult;
	if player.cash < base_cost {
		msg.channel_id.send_message(&ctx.http, |m| m.content(&format!("You don't have enough... You need **${:.2}** more", base_cost - player.cash))).await?;
		return Ok(());
	}
	let total_cost = base_cost * amount as f64;
	let amount = vec![(total_cost / base_cost).floor(), (player.cash / base_cost).floor()]
		.into_iter()
		.reduce(f64::min)
		.unwrap() as i32; // Either the most they can afford or the amount they wanted.
	player.cash -= base_cost * amount as f64;
	*player.packs.entry(set.id()).or_insert(0) += (amount * pack_count) as i64;
	player.packs_bought += (amount * pack_count) as i64;
	msg.channel_id.send_message(&ctx.http, |m| m.content(&format!("You bought {} **{}** packs!", amount * pack_count, set.name))).await?;
	let mut player_packs = Document::new();
	for (set_id, amt) in player.packs.iter() {
		player_packs.insert(set_id, amt.clone());
	}
	player::update_player(
		&player,
		doc! {
			"$set": {
				"cash": &player.cash,
				"packs_bought": &player.packs_bought,
				"packs": player_packs
			}
		}
	).await;

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
				
					m.content(format!("Your daily resets **{}**", local_timer.format("%h %d %H:%m")))
				})
			.await
			.unwrap();
		return Ok(());
	}
	let mut update = Document::new();
	let r: i64 = rand::thread_rng().gen_range(0..100);
	if r <= 2 {
		player.daily_packs += 50;
		update.insert("daily_packs", player.daily_packs);
		msg.reply(&ctx.http, "***WOAH!*** Your daily packs were reset!").await?;
	} else {
		let cash: i64 = rand::thread_rng().gen_range(5..=20);
		player.cash += cash as f64;
		player.total_cash += cash as f64;
		update.insert("cash", player.cash);
		update.insert("total_cash", player.total_cash);
		msg.reply(&ctx.http, format!("You got **${:.2}**", cash as f64)).await?;
	}
	player.daily_reset = Utc::now() + Duration::days(1);
	update.insert("daily_reset", player.daily_reset);
	player::update_player(&player, doc!{ "$set": update }).await;

	Ok(())
}

// #[command("quiz")]
// async fn quiz_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	

// 	Ok(())
// }

#[command("savelist")]
#[aliases("sl", "favourite", "favorite", "fv")]
#[sub_commands(savelist_add, savelist_clear, savelist_remove)]
async fn savelist_main(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	let mut cards = card::get_multiple_cards_by_id(player.savelist.clone()).await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "You have no cards in your savelist! Use **.savelist add <card id>** to add a card\nOr use the :floppy_disk: emoji when scrolling through cards!").await?;
	} else {
		cards.sort_by(|c1, c2| c1.name.cmp(&c2.name));
		card_paginated_embeds(ctx, msg, cards, player).await?;
	}

	Ok(())
}

#[command("add")]
#[aliases("a", "+")]
async fn savelist_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let card_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if card_id == "" {
		msg.reply(&ctx.http, "No card provided").await?;
		return Ok(());
	}
	let mut player = player::get_player(msg.author.id.0).await;
	let card = card::get_card(&card_id).await;
	if player.savelist.contains(&card_id) {
		msg.reply(&ctx.http, format!("**{}** is already in your savelist", card.name)).await?;
		return Ok(());
	}
	msg.reply(&ctx.http, format!("**{}** added to your savelist", card.name)).await?;
	player.savelist.push(card_id);
	player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;

	Ok(())
}

#[command("remove")]
#[aliases("r", "-")]
async fn savelist_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let card_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if card_id == "" {
		msg.reply(&ctx.http, "No card provided").await?;
		return Ok(());
	}
	let mut player = player::get_player(msg.author.id.0).await;
	let card = card::get_card(&card_id).await;
	if !player.savelist.contains(&card_id) {
		msg.reply(&ctx.http, format!("**{}** is not in your savelist", card.name)).await?;
		return Ok(());
	}
	msg.reply(&ctx.http, format!("**{}** removed from your savelist", card.name)).await?;
	let index = player.savelist.clone().iter().position(|c| c == &card_id).unwrap();
	player.savelist.remove(index);
	player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;

	Ok(())
}

#[command("clear")]
async fn savelist_clear(ctx: &Context, msg: &Message) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	player.savelist = vec![];
	player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;
	msg.reply(&ctx.http, "Your savelist has been cleared").await?;

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

#[command("with")]
#[aliases("w")]
async fn trade_with(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	let tradee_mention = msg.mentions.iter().nth(0);
	match tradee_mention {
		Some(x) => {
			if x.id == msg.author.id {
				msg.reply(&ctx.http, "You can't trade with yourself...").await?;
				return Ok(());
			}
		},
		None => {
			msg.reply(&ctx.http, "You didn't choose to trade with anyone").await?;
			return Ok(());
		}
	}
	args.advance();
	let trade_str = args.remains();
	match trade_str {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You didn't choose to trade anything").await?;
			return Ok(());
		}
	}
	let trade_offer = trade::Trade::from_trade_str(trade_str.unwrap());
	println!("{:?}", &trade_offer);
	if !trade_offer.player_has_all(&player) {
		msg.reply(&ctx.http, "You don't own all of what you're offering!").await?;
		return Ok(());
	}
	let tradee_mention = tradee_mention.unwrap();
	let mut tradee = player::get_player(tradee_mention.id.0).await;
	let counteroffer_ask = msg
		.channel_id
		.send_message(&ctx.http, |m| {m
			.content(
				format!("<@{}> what do you want to trade for **{}**?", tradee_mention.id.0, trade_str.unwrap())
			)
		})
		.await?;
	if let Some(tradee_reply) = &tradee_mention.await_reply(&ctx).timeout(StdDuration::from_secs(120)).await {
		let counteroffer = trade::Trade::from_trade_str(&tradee_reply.content);
		if !counteroffer.player_has_all(&tradee) {
			counteroffer_ask.reply(&ctx.http, "You don't own all of what you're offering!").await?;
			return Ok(());
		}
		// See if the player accepts the trade
		let _ = msg
			.channel_id
			.send_message(&ctx.http, |m| {m
				.content(
					format!("<@{}> do you accept the trade **{}** for **{}**? (y/n)", msg.author.id.0, trade_str.unwrap(), tradee_reply.content)
				)
			})
			.await?;
		if let Some(accept_reply) = &msg.author.await_reply(&ctx).timeout(StdDuration::from_secs(30)).await {
			if accept_reply.content.to_lowercase() != "y" {
				counteroffer_ask.reply(&ctx.http, "Your trade has been denied.").await?;
				return Ok(());
			}
		} else {
			counteroffer_ask.reply(&ctx.http, "Your trade has been denied.").await?;
			return Ok(());
		}
		// Remove items from trade_offer from player and add them to tradee
		player.cash -= trade_offer.cash;
		tradee.cash += trade_offer.cash;
		for (card_id, amount) in trade_offer.cards {
			*player.cards.entry(card_id.clone()).or_insert(0) -= amount;
			if *player.cards.entry(card_id.clone()).or_insert(0) == 0 {
				player.cards.remove(&card_id);
			}
			*tradee.cards.entry(card_id.clone()).or_insert(0) += amount;
		}
		for (pack_id, amount) in trade_offer.packs {
			*player.packs.entry(pack_id.clone()).or_insert(0) -= amount;
			if *player.packs.entry(pack_id.clone()).or_insert(0) == 0 {
				player.packs.remove(&pack_id);
			}
			*tradee.packs.entry(pack_id.clone()).or_insert(0) += amount;
		}
		// Remove items from counteroffer from tradee and add them to player
		player.cash += counteroffer.cash;
		tradee.cash -= counteroffer.cash;
		for (card_id, amount) in counteroffer.cards {
			*tradee.cards.entry(card_id.clone()).or_insert(0) -= amount;
			if *tradee.cards.entry(card_id.clone()).or_insert(0) == 0 {
				tradee.cards.remove(&card_id);
			}
			*player.cards.entry(card_id.clone()).or_insert(0) += amount;
		}
		for (pack_id, amount) in counteroffer.packs {
			*tradee.packs.entry(pack_id.clone()).or_insert(0) -= amount;
			if *tradee.packs.entry(pack_id.clone()).or_insert(0) == 0 {
				tradee.packs.remove(&pack_id);
			}
			*player.packs.entry(pack_id.clone()).or_insert(0) += amount;
		}
		// Update the player
		let mut player_update = Document::new();
		player_update.insert("cash", player.cash);
		let mut player_cards_doc = Document::new();
		for (crd, amt) in player.cards.iter() {
			player_cards_doc.insert(crd, amt);
		}
		player_update.insert("cards", player_cards_doc);
		let mut player_packs_doc = Document::new();
		for (pck, amt) in player.packs.iter() {
			player_packs_doc.insert(pck, amt);
		}
		player_update.insert("packs", player_packs_doc);
		player::update_player(&player, doc! { "$set": player_update }).await;
		// Update the tradee
		let mut tradee_update = Document::new();
		tradee_update.insert("cash", tradee.cash);
		let mut tradee_cards_doc = Document::new();
		for (crd, amt) in tradee.cards.iter() {
			tradee_cards_doc.insert(crd, amt);
		}
		tradee_update.insert("cards", tradee_cards_doc);
		let mut tradee_packs_doc = Document::new();
		for (pck, amt) in tradee.packs.iter() {
			tradee_packs_doc.insert(pck, amt);
		}
		tradee_update.insert("packs", tradee_packs_doc);
		player::update_player(&tradee, doc! { "$set": tradee_update }).await;
		msg
			.reply(&ctx.http, format!("Traded <@{}> **{}** for **{}**", tradee_mention.id.0, trade_str.unwrap(), tradee_reply.content)).await?;
	} else {
		msg.reply(&ctx.http, format!("<@{}> didn't reply \\:(", tradee_mention.id.0)).await?;
		return Ok(());
	}
	

	Ok(())
}

#[command("gamecorner")]
#[aliases("gc", "game", "corner", "gamec")]
#[sub_commands(game_corner_payouts, game_corner_slots, game_corner_tokens_main)]
async fn game_corner_main(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
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

						f.text(&format!("Resets {}", local_timer.format("%h %d %H:%m")))
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
	let mut desc = String::from("Here are the token payouts for the slot machines\n");
	desc.push_str("<:GameCorner:967522912242384906><:GameCorner:967522912242384906><:GameCorner:967522912242384906> **500**\n");
	desc.push_str("<:GameCorner:967522912166903858><:GameCorner:967522912166903858><:GameCorner:967522912166903858> **200**\n");
	desc.push_str("<:GameCorner:967522912196239510><:GameCorner:967522912196239510><:GameCorner:967522912196239510> **120**\n");
	desc.push_str("<:GameCorner:967522912275922995><:GameCorner:967522912275922995><:GameCorner:967522912275922995> **80**\n");
	desc.push_str("<:GameCorner:967522912154296410><:GameCorner:967522912154296410><:GameCorner:967522912154296410> **50**\n");
	desc.push_str("<:GameCorner:967522912229793882><:GameCorner:967522912229793882><:GameCorner:967522912229793882> **30**\n");
	desc.push_str("<:GameCorner:967522912166871080><:GameCorner:967522912166871080><:GameCorner:967522912166871080> **15**\n");
	desc.push_str("<:GameCorner:967522912166871080><:GameCorner:967522912166871080><:GameCorner:967591653135228988> **5**");
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
	let mut player = player::get_player(msg.author.id.0).await;
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
	let slots = slot::Slot::new(amount);
	let mut roll_displays = vec![];
	for roll in slots.rolls {
		let reward = roll.reward();
		player.tokens += reward;
		player.total_tokens += reward;
		player.slots_rolled += 1;
		player.daily_slots -= 1;
		match (roll.slot1.as_str(), roll.slot2.as_str(), roll.slot3.as_str()) {
			("7", "7", "7") => player.jackpots += 1,
			("7", "7", "R") => player.boofs += 1,
			_ => ()
		}
		roll_displays.push(roll.reward_display());
	}
	msg.reply(&ctx.http, roll_displays.join("\n")).await?;
	let mut player_update = Document::new();
	player_update.insert("tokens", player.tokens);
	player_update.insert("total_tokens", player.total_tokens);
	player_update.insert("slots_rolled", player.slots_rolled);
	player_update.insert("daily_slots", player.daily_slots);
	player_update.insert("jackpots", player.jackpots);
	player_update.insert("boofs", player.boofs);
	player::update_player(&player, doc! { "$set": player_update }).await;

	Ok(())
}

#[command("tokenshop")]
#[aliases("ts", "tokens")]
#[sub_commands(game_corner_tokens_buy, game_corner_tokens_convert)]
async fn game_corner_tokens_main(ctx: &Context, msg: &Message) -> CommandResult {
	let token_shop = slot::get_token_shop().await;
	let player = player::get_player(msg.author.id.0).await;
	let embed = token_shop.embed_with_player(player).await;
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
	let token_shop = slot::get_token_shop().await;
	if !(1..=5).contains(&selection) {
		msg.channel_id.send_message(&ctx.http, |m| m.content("A selection was not made.")).await?;
		return Ok(());
	}
	let amount = match args.single::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let mut update = Document::new();
	let mut player = player::get_player(msg.author.id.0).await;
	if selection <= 3 {
		let set = get_set(token_shop.sets.get((selection - 1) as usize).unwrap()).await.unwrap();
		let base_cost = slot::to_tokens(set.pack_price());
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
			4 => card::get_card(&token_shop.rare_card).await,
			_ => card::get_card(&token_shop.rainbow_card).await
		};
		let base_cost = slot::to_tokens(card.price) * 10;
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
	player::update_player(&player, doc! { "$set": update }).await;

	Ok(())
}

#[command("convert")]
#[aliases("c")]
async fn game_corner_tokens_convert(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let amount = match args.single::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let mut player = player::get_player(msg.author.id.0).await;
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
	player::update_player(&player, doc!{ "$set": update}).await;

	Ok(())
}


// ADMIN COMMANDS (FOR TESTING)
#[command("admin")]
#[sub_commands(admin_show_pack, admin_add_cash, admin_mock_slot, admin_add_tokens)]
#[checks(BotTest)]
async fn admin_main() -> CommandResult {
	Ok(())
}

#[command("pack")]
#[checks(BotTest)]
async fn admin_show_pack(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let set_id = args.find::<String>().unwrap();
	let amount = match args.find::<i32>() {
		Ok(x) => x as usize,
		Err(_) => 1usize
	};
	let pack = packs::Pack::from_set_id(set_id.as_str(), amount).await.unwrap();
	paginated_embeds(ctx, msg, pack.cards).await.unwrap();

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
		roll_displays.push(roll.reward_display());
	}
	let content = roll_displays.join("\n");

	msg.reply(&ctx.http, content).await?;

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
			if player.daily_packs < 50 {
				player.daily_packs = 50;
				update.insert("daily_packs", player.daily_packs);
			}
			player.daily_slots = 10;
			update.insert("daily_slots", player.daily_slots);
			player::update_player(&player, doc!{"$set": update }).await;
		}
		timers::update_timer(&timer).await;
	}
}

/* Tasks
 * v1.3.0
 * 	FIX: api_call needs to grab more than 250 items
 * 	Caching
 * 		Use the database? create a cachedcards, cachedsets, and cachedtimers collection
 * 		Refresh them daily for price updates
 * 			Learn to do a second loop on a different timer (hourly with the cachedtimer.reset)
 * v1.4.0
 * 	Add upgrades for
 * 		daily time reset
 * 		slot reward multiplier
 * 		store doscount
 * 		token shop discount
 * 		daily pack amount
 * 		daily reward multiplier
 * 		quiz reward multiplier
 * 	Add global cash sinks (buys for all players) (aka expensive)
 * 		daily reset
 * 		token shop refresh
 * 		store refresh
 * 		pack reset
 * 		slot reset
 * 	Add quiz
 * 		Make it so that quiz only awards money for 5 every 2 hours but people can keep playing
 * Misc
 * 	Add a help command (Okay shits not working so I don't care right now.)
 *	 	Add usage macro to all the commands and sub commands
 * 	Add a changelog command that DMs the user the patch note they want
 * 		Add the current version to the activity
*/

/* Admin commands
 * cache
 * resetpacks
 * resetquiz
*/

/*
 * TODOS:
 * 	Add .buy command as a shortcut to .store buy
 * 	Learn image manipulation to make the .quiz commands
*/