use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
		Document
	}, 
	Collection
};
use serde::{Serialize, Deserialize};
use super::{
	sets,
	card,
	player,
	binder_paginated_embeds,
	Idable,
	CardInfo,
	player_card,
	HasSet,
	card_paginated_embeds
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
		},
	},
	utils::{
		Colour
	},
	prelude::*
};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binder {
	pub set: String,
	pub cards: Vec<String>
}

impl Binder {
	pub fn empty() -> Self {
		Self {
			set: String::from(""),
			cards: vec![]
		}
	}

	pub fn from_set_id(set: String) -> Self {
		Self {
			set,
			cards: vec![]
		}
	}

	pub fn to_doc(&self) -> Document {
		let mut d = Document::new();
		d.insert("set", &self.set);
		d.insert("cards", &self.cards);

		d
	}

	pub async fn is_complete(&self) -> bool {
		let set = sets::get_set(&self.set).await.unwrap();
		let cards = card::get_cards_by_set(&set).await;

		cards.len() == self.cards.len()
	}

}

#[command("binder")]
#[aliases("b")]
#[sub_commands(binder_start, binder_add, binder_showcase, binder_missing)]
async fn binder_main(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	match player.current_binder.set.as_str() {
		"" => {
			msg.reply(&ctx.http, "You don't have a binder started! Use **.binder start <set id>** to start one!").await?;
		},
		_ => binder_paginated_embeds(ctx, msg, player, false).await?,
	}

	Ok(())
}

#[command("start")]
#[aliases("st")]
async fn binder_start(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
	let mut player = player::get_player(msg.author.id.0).await;
	if player.current_binder.set.as_str() != "" {
		let current_binder_set = sets::get_set(&player.current_binder.set).await.unwrap();
		msg.reply(&ctx.http, format!("You already started a binder for **{}**", current_binder_set.name)).await?;
		return Ok(());
	}
	if player.completed_binders.contains(&set.id()) {
		msg.reply(&ctx.http, format!("You already completed a binder for **{}**", set.name)).await?;
		return Ok(());
	}
	let binder = Binder::from_set_id(set.id());
	let _ = msg.reply(&ctx.http, format!("Once a binder has been started, you ***CAN'T*** start another until it's complete.\nDo you want to start a binder for **{}** (y/n)", set.name)).await?;
	if let Some(confirmation_reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		if confirmation_reply.content.to_lowercase() != "y" {
			msg.reply(&ctx.http, "You did not start the binder.").await?;
			return Ok(());
		}
	} else {
		msg.reply(&ctx.http, "You did not start the binder.").await?;
		return Ok(());
	}
	// Player said "y" to get here
	player.current_binder = binder;
	let mut player_update = Document::new();
	player_update.insert("current_binder", player.current_binder.to_doc());
	msg.reply(&ctx.http, format!("You started the binder for **{}**", set.name)).await?;
	player::update_player(&player, doc! { "$set": player_update }).await;

	Ok(())
}

#[command("add")]
#[aliases("a", "+")]
#[sub_commands(binder_add_bulk)]
async fn binder_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let card_id = match args.find::<String>() {
		Ok(x) => x,
		Err(_) => String::from("")
	};
	if card_id.as_str() == "" {
		msg.reply(&ctx.http, "No card was provided.").await?;
		return Ok(());
	}
	let card = card::get_card(&card_id).await;
	let mut player = player::get_player(msg.author.id.0).await;
	if player.current_binder.set.as_str() == "" {
		msg.reply(&ctx.http, "You don't have a binder started! Use **.binder start <set id>** to start one!").await?;
		return Ok(());
	}
	if !player.cards.contains_key(&card.card_id()) {
		msg.reply(&ctx.http, "You don't own that card.").await?;
		return Ok(());
	}
	let current_binder_set = sets::get_set(&player.current_binder.set).await.unwrap();
	if current_binder_set.id() != card.set.id() {
		msg.reply(&ctx.http, "That card doesn't belong in this binder.").await?;
		return Ok(());
	}
	if player.current_binder.cards.contains(&card.id()) {
		msg.reply(&ctx.http, "That card is already in the binder").await?;
		return Ok(());
	}
	let mut player_update = Document::new();
	if player.savelist.contains(&card.card_id()) && player.cards.get(&card.card_id()).unwrap() == &1 {
		let _ = msg.reply(&ctx.http, format!("**{}** is in your savelist, and you only have 1 left. Do you want to add it to your binder? (y/n)", card.name)).await?;
		if let Some(confirmation_reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
			if confirmation_reply.content.to_lowercase() != "y" {
				msg.reply(&ctx.http, "You didn't add the card to your binder.").await?;
				return Ok(());
			}
		} else {
			msg.reply(&ctx.http, "You didn't add the card to your binder.").await?;
			return Ok(());
		}
	}
	*player.cards.entry(card.card_id()).or_insert(0) -= 1;
	if *player.cards.entry(card.card_id()).or_insert(0) == 0 {
		player.cards.remove(&card.card_id());
	}
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	player_update.insert("cards", player_cards);
	player.current_binder.cards.push(card.card_id());
	if player.current_binder.is_complete().await {
		player.completed_binders.push(player.current_binder.set);
		player.current_binder = Binder::empty();
		player_update.insert("completed_binders", player.completed_binders.clone());
		msg.reply(&ctx.http, format!("You completed the **{}** binder!", current_binder_set.name)).await?;
	} else {
		msg.reply(&ctx.http, format!("You added **{}** to your binder!", card.name)).await?;
	}
	player_update.insert("current_binder", player.current_binder.to_doc());
	player::update_player(&player, doc! { "$set": player_update }).await;

	Ok(())
}

#[command("bulk")]
#[aliases("b")]
async fn binder_add_bulk(ctx: &Context, msg: &Message) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	if player.current_binder.set.as_str() == "" {
		msg.reply(&ctx.http, "You don't have a binder started! Use **.binder start <set id>** to start one!").await?;
		return Ok(());
	}
	let cards = player_card::player_cards(player.cards.clone()).await;
	let binder_cards = cards
		.iter()
		.filter(|c| c.set().id() == player.current_binder.set && !player.current_binder.cards.contains(&c.card_id()))
		.map(|c| c.to_owned())
		.collect::<Vec<player_card::PlayerCard>>();
	if binder_cards.len() <= 0 {
		msg.reply(&ctx.http, "No cards to add!").await?;
		return Ok(());
	}
	let mut player_update = Document::new();
	for binder_card in binder_cards.clone() {
		*player.cards.entry(binder_card.card_id()).or_insert(0) -= 1;
		if *player.cards.entry(binder_card.card_id()).or_insert(0) == 0 {
			player.cards.remove(&binder_card.card_id());
		}
		player.current_binder.cards.push(binder_card.card_id());
	}
	let mut player_cards = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards.insert(crd, amt);
	}
	player_update.insert("cards", player_cards);
	if player.current_binder.is_complete().await {
		let current_binder_set = sets::get_set(&player.current_binder.set).await.unwrap();
		player.completed_binders.push(player.current_binder.set);
		player.current_binder = Binder::empty();
		player_update.insert("completed_binders", player.completed_binders.clone());
		msg.reply(&ctx.http, format!("You completed the **{}** binder!", current_binder_set.name)).await?;
	} else {
		msg.reply(&ctx.http, format!("You added **{}** cards to your binder!", binder_cards.len())).await?;
	}
	player_update.insert("current_binder", player.current_binder.to_doc());
	player::update_player(&player, doc! { "$set": player_update }).await;
	card_paginated_embeds(ctx, msg, binder_cards, player).await?;

	Ok(())
}

#[command("showcase")]
#[aliases("sc")]
async fn binder_showcase(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	if player.completed_binders.len() == 0 {
		msg.reply(&ctx.http, "You have no completed binders!").await?;
		return Ok(());
	}
	let mut desc = String::from("");
	for completed_binder in player.completed_binders {
		let set = sets::get_set(&completed_binder).await.unwrap();
		desc.push_str(&format!(":first_place: **{}** (_{}_)\n", set.name, set.id()));
	}
	msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e
					.title("Your Completed Binders")
					.description(&desc)
					.colour(Colour::from_rgb(255, 50, 20))
			})
		})
		.await?;

	Ok(())
}

#[command("missing")]
#[aliases("m")]
async fn binder_missing(ctx: &Context, msg: &Message) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	if player.current_binder.set.as_str() == "" {
		msg.reply(&ctx.http, "You don't have a binder started! Use **.binder start <set id>** to start one!").await?;
		return Ok(());
	}
	binder_paginated_embeds(ctx, msg, player, true).await?;

	Ok(())
}
