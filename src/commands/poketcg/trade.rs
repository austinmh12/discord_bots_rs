use crate::{
	player::{
		Player,
		get_player,
		update_player
	},
};
use mongodb::bson::{Document, doc};
use std::time::Duration as StdDuration;
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
	model::{
		channel::{
			Message,
		},
	},
	prelude::*
};

#[derive(Debug)]
pub struct Trade {
	pub cash: f64,
	pub cards: Vec<(String, i64)>,
	pub packs: Vec<(String, i64)>
}

impl Trade {
	pub fn from_trade_str(trade_str: &str) -> Self {
		let offers = trade_str.split("/").collect::<Vec<&str>>();
		let mut cash = 0.00;
		let mut cards = vec![];
		let mut packs = vec![];
		for offer in offers {
			if offer.contains("$") {
				let amt = offer
					.replace("$", "")
					.parse::<f64>()
					.unwrap_or(0.00);
				cash += amt;
			} else if offer.contains("-") {
				let card_amount = offer
					.split(":")
					.collect::<Vec<&str>>();
				let card = String::from(card_amount[0]);
				if card_amount.len() == 1 {
					cards.push((card, 1));
				} else {
					let amt = card_amount[1].parse::<i64>().unwrap_or(1);
					cards.push((card, amt));
				}
			} else {
				let pack_amount = offer
					.split(":")
					.collect::<Vec<&str>>();
				let pack = String::from(pack_amount[0]);
				if pack_amount.len() == 1 {
					packs.push((pack, 1));
				} else {
					let amt = pack_amount[1].parse::<i64>().unwrap_or(1);
					packs.push((pack, amt));
				}
			}
		}

		Self {
			cash,
			cards,
			packs
		}
	}

	pub fn player_has_all(&self, player: &Player) -> bool {
		if player.cash < self.cash {
			return false;
		}
		for (card_id, amt) in &self.cards {
			if player.cards.contains_key(card_id) {
				if player.cards.get(card_id).unwrap() < &amt {
					return false;
				}
			} else {
				return false;
			}
		}
		for (pack_id, amt) in &self.packs {
			if player.packs.contains_key(pack_id) {
				if player.packs.get(pack_id).unwrap() < &amt {
					return false;
				}
			} else {
				return false;
			}
		}

		true
	}
}

#[command("with")]
#[aliases("w")]
async fn trade_with(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player = get_player(msg.author.id.0).await;
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
	let trade_offer = Trade::from_trade_str(trade_str.unwrap());
	println!("{:?}", &trade_offer);
	if !trade_offer.player_has_all(&player) {
		msg.reply(&ctx.http, "You don't own all of what you're offering!").await?;
		return Ok(());
	}
	let tradee_mention = tradee_mention.unwrap();
	let mut tradee = get_player(tradee_mention.id.0).await;
	let counteroffer_ask = msg
		.channel_id
		.send_message(&ctx.http, |m| {m
			.content(
				format!("<@{}> what do you want to trade for **{}**?", tradee_mention.id.0, trade_str.unwrap())
			)
		})
		.await?;
	if let Some(tradee_reply) = &tradee_mention.await_reply(&ctx).timeout(StdDuration::from_secs(120)).await {
		let counteroffer = Trade::from_trade_str(&tradee_reply.content);
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
		update_player(&player, doc! { "$set": player_update }).await;
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
		update_player(&tradee, doc! { "$set": tradee_update }).await;
		msg
			.reply(&ctx.http, format!("Traded <@{}> **{}** for **{}**", tradee_mention.id.0, trade_str.unwrap(), tradee_reply.content)).await?;
	} else {
		msg.reply(&ctx.http, format!("<@{}> didn't reply \\:(", tradee_mention.id.0)).await?;
		return Ok(());
	}
	
	Ok(())
}