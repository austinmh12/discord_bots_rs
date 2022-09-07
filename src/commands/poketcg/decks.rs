use std::collections::HashMap;
use std::time::Duration;

use super::*;
use async_trait::async_trait;
use futures::TryStreamExt;
use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		Document,
		oid::ObjectId,
	}, 
	Collection
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

use crate::{
	commands::get_client,
	player::{
		get_player,
		update_player,
		Player
	},
	card::{
		get_multiple_cards_by_id,
		get_card,
		Card
	},
	commands::poketcg::Scrollable,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deck {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub discord_id: i64,
	pub name: String,
	pub cards: HashMap<String, i64>,
	pub display_card: String
}

impl Deck {
	pub fn empty(discord_id: i64, name: String) -> Self {
		Self {
			id: None,
			discord_id,
			name,
			cards: HashMap::new(),
			display_card: "".into()
		}
	}

	pub fn is_valid(&self) -> bool {
		self.cards.values().sum::<i64>() == 60
	}

	pub async fn get_cards(&self) -> Vec<Card> {
		let card_ids = self.cards.keys().into_iter().map(|k| k.into()).collect::<Vec<String>>();
		let cards = get_multiple_cards_by_id(card_ids).await;

		cards
	}

	pub async fn get_display_image(&self, ctx: &Context) -> String {
		let image: String = match self.display_card.as_str() {
			"" => {
				let mut ret = "".into();
				let cards = self.get_cards().await;
				if cards.len() != 0 {
					ret = cards.into_iter().nth(0).unwrap().image;
				}

				ret
			},
			_ => {
				let card = get_card(ctx, &self.display_card).await;

				card.image
			}
		};

		image
	}
}

impl PaginateEmbed for Deck {
	fn embed(&self) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		ret
			.title(&self.name)
			.colour(Colour::from_rgb(255, 50, 20));

		ret
	}
}

#[async_trait]
impl Scrollable for Vec<Deck> {
	async fn scroll_through(&self, ctx: &Context, msg: &Message) -> Result<(), String> {
		let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
		let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
		let pokemon_card = ReactionType::try_from("<:poketcg:965802882433703936>").expect("No TCG Back");
		let decks = &self.clone();
		let embeds = self.iter().map(|e| e.embed()).collect::<Vec<_>>();
		let mut idx: i16 = 0;
		let mut deck = decks.into_iter().nth(idx as usize).unwrap();
		let mut deck_display = deck.get_display_image(ctx).await;
		let mut message = msg
			.channel_id
			.send_message(&ctx.http, |m| {
				let mut cur_embed = embeds[idx as usize].clone();
				if embeds.len() > 1 {
					cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
				}
				cur_embed.image(deck_display);
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
						let deck = decks.into_iter().nth(idx as usize).unwrap();
						let cards = deck.get_cards().await;
						message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");	
						cards.scroll_through(ctx, msg).await?
					},
					_ => continue
				};
			} else {
				message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
				break;
			}
			deck = decks.into_iter().nth(idx as usize).unwrap();
			deck_display = deck.get_display_image(ctx).await;
			message.edit(&ctx, |m| {
				let mut cur_embed = embeds[idx as usize].clone();
				if embeds.len() > 1 {
					cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
				}
				cur_embed.image(deck_display);
				m.set_embed(cur_embed);

				m
			}).await.unwrap();
		}

		Ok(())
	}
}

async fn get_deck_collection() -> Collection<Deck> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Deck>("decks");

	collection
}

pub async fn add_deck(deck: &Deck) {
	let deck_collection = get_deck_collection().await;
	deck_collection
		.insert_one(deck, None)
		.await
		.unwrap();
}

pub async fn get_decks_by_player(discord_id: i64) -> Vec<Deck> {
	let deck_collection = get_deck_collection().await;
	let decks = deck_collection
		.find(doc! { "discord_id": discord_id }, None)
		.await
		.unwrap()
		.try_collect::<Vec<Deck>>()
		.await
		.unwrap();

	decks
}

pub async fn get_deck(discord_id: i64, name: String) -> Option<Deck> {
	let deck_collection = get_deck_collection().await;
	let deck = deck_collection
		.find_one(doc! { "discord_id": discord_id, "name": name }, None)
		.await
		.unwrap();

	deck
}

pub async fn update_deck(deck: &Deck, update: Document) {
	let deck_collection = get_deck_collection().await;
	deck_collection
		.update_one(
			doc! { "_id": &deck.id.unwrap() },
			update,
			None
		)
		.await
		.unwrap();
}

pub async fn delete_deck(deck: &Deck) {
	let deck_collection = get_deck_collection().await;
	deck_collection
		.delete_one(
			doc! { "_id": &deck.id.unwrap() },
			None
		)
		.await
		.unwrap();
}

#[command("decks")]
#[aliases("dks")]
async fn decks_command(ctx: &Context, msg: &Message) -> CommandResult {
	let player = get_player(msg.author.id.0).await;
	let decks = get_decks_by_player(player.discord_id).await;
	match decks.len() {
		0 => {
			msg.reply(&ctx.http, "You don't have any decks! Use **.deck create <name>** to create one!").await?;
		},
		_ => decks.scroll_through(ctx, msg).await?
	}

	Ok(())
}

#[command("deck")]
#[aliases("dk")]
#[sub_commands(deck_view, deck_create, deck_delete, deck_add, deck_remove, deck_energy_main, deck_display)]
async fn deck_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available deck commands:
	**.decks** to see all your current decks.
	**.deck view <name>** to view a specific deck
	**.deck create <name>** to create a new deck.
	**.deck delete <name>** to delete a deck that you've created.
	**.deck add <name> [<cardID:amount>/...]** to add cards to a deck.
	**.deck remove <name> [<cardID:amount>/...]** to remove cards from a deck.
	**.deck energy add <name> <type> [amount - Default: 1]** to add a basic energy to a deck.
	**.deck energy remove <name> <type> [amount - Default: 1]** to remove a basic energy from a deck.
	**.deck display <name> <cardID>** to set the display card of the deck";
	msg
		.channel_id
		.send_message(&ctx.http, |m| m.content(content))
		.await?;

	Ok(())
}

#[command("view")]
#[aliases("v")]
async fn deck_view(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let deck_name = args.rest().to_lowercase();
	let player = get_player(msg.author.id.0).await;
	if deck_name == String::from("") {
		return decks_command(ctx, msg, args).await;
	}
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let deck = deck.unwrap();
	let card_ids = deck.cards
		.keys()
		.into_iter()
		.map(|c| c.into())
		.collect::<Vec<String>>();
	let cards = get_multiple_cards_by_id(card_ids).await;
	cards.scroll_through(ctx, msg).await?;
	
	Ok(())
}

#[command("create")]
#[aliases("c")]
async fn deck_create(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let deck_name = args.rest().to_lowercase();
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let player = get_player(msg.author.id.0).await;
	match get_deck(player.discord_id, deck_name.clone()).await {
		Some(_) => {
			msg.reply(&ctx.http, "You already have a deck with that name!").await?;
			return Ok(());
		},
		None => ()
	}
	let deck = Deck::empty(player.discord_id, deck_name.clone());
	add_deck(&deck).await;
	msg.reply(&ctx.http, format!("You created the deck **{}**", deck_name)).await?;

	Ok(())
}

#[command("delete")]
#[aliases("d")]
async fn deck_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let deck_name = args.rest().to_lowercase();
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let mut player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let deck = deck.unwrap();
	let _ = msg.reply(&ctx.http, format!("Are you sure you want to delete this deck?\nOnce you delete **{}** it's gone forever. (y/n)", deck.name)).await?;
	if let Some(confirmation_reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		if confirmation_reply.content.to_lowercase() != "y" {
			msg.reply(&ctx.http, format!("You did not delete **{}**.", deck.name)).await?;
			return Ok(());
		}
	} else {
		msg.reply(&ctx.http, format!("You did not delete **{}**.", deck.name)).await?;
		return Ok(());
	}
	// Player said "y" to get here
	for (crd, amt) in deck.cards.iter() {
		if vec!["col1-94", "col1-93", "col1-89", "col1-88", "col1-91", "col1-95", "col1-92", "col1-90"].contains(&crd.as_str()) {
			continue;
		}
		*player.cards.entry(crd.clone()).or_insert(0) += amt;
	}
	// Update the player
	let mut player_update = Document::new();
	let mut player_cards_update = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards_update.insert(crd, amt);
	}
	player_update.insert("cards", player_cards_update);
	update_player(&player, doc! { "$set": player_update }).await;
	delete_deck(&deck).await;
	msg.reply(&ctx.http, format!("You deleted **{}**", deck.name)).await?;

	Ok(())
}

#[command("add")]
#[aliases("a")]
async fn deck_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let deck_name = args.find::<String>().unwrap_or(String::from(""));
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let card_str = args.rest();
	if card_str == "" {
		msg.reply(&ctx.http, "You didn't provide cards to add.").await?;
		return Ok(());
	}
	let mut player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let mut deck = deck.unwrap();
	let deckcards = DeckCards::from_card_str(card_str);
	if !deckcards.player_has_all(&player) {
		msg.reply(&ctx.http, "You don't own all of what you're putting in the deck!").await?;
		return Ok(());
	}
	if !deckcards.is_valid_addition(&deck) {
		// Maybe update this to list what's not valid
		msg.reply(&ctx.http, "You have invalid additions to this deck!").await?;
		return Ok(());
	}
	for (card_id, amt) in deckcards.cards {
		*player.cards.entry(card_id.clone()).or_insert(0) -= amt;
		if *player.cards.entry(card_id.clone()).or_insert(0) == 0 {
			player.cards.remove(&card_id);
		}
		*deck.cards.entry(card_id.clone()).or_insert(0) += amt;
	}
	player.cards.retain(|_, v| *v > 0);
	// Update the player
	let mut player_update = Document::new();
	let mut player_cards_update = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards_update.insert(crd, amt);
	}
	player_update.insert("cards", player_cards_update);
	update_player(&player, doc! { "$set": player_update }).await;

	// Update the deck
	let mut deck_update = Document::new();
	let mut deck_card_update = Document::new();
	for (crd, amt) in deck.cards.iter() {
		deck_card_update.insert(crd, amt);
	}
	deck_update.insert("cards", deck_card_update);
	update_deck(&deck, doc! { "$set": deck_update }).await;

	msg.reply(&ctx.http, format!("You added **{}** to **{}**", card_str, deck.name)).await?;

	Ok(())
}

#[command("remove")]
#[aliases("r")]
async fn deck_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let deck_name = args.find::<String>().unwrap_or(String::from(""));
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let card_str = args.rest();
	if card_str == "" {
		msg.reply(&ctx.http, "You didn't provide cards to remove.").await?;
		return Ok(());
	}
	let mut player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let mut deck = deck.unwrap();
	let deckcards = DeckCards::from_card_str(card_str);
	if !deckcards.deck_has_all(&deck) {
		msg.reply(&ctx.http, "The deck doesn't have all of what you're removing!").await?;
		return Ok(());
	}
	for (card_id, amt) in deckcards.cards {
		if vec!["col1-94", "col1-93", "col1-89", "col1-88", "col1-91", "col1-95", "col1-92", "col1-90"].contains(&card_id.as_str()) {
			continue;
		}
		*deck.cards.entry(card_id.clone()).or_insert(0) -= amt;
		if *deck.cards.entry(card_id.clone()).or_insert(0) == 0 {
			deck.cards.remove(&card_id);
		}
		*player.cards.entry(card_id.clone()).or_insert(0) += amt;
	}
	deck.cards.retain(|_, v| *v > 0);
	// Update the player
	let mut player_update = Document::new();
	let mut player_cards_update = Document::new();
	for (crd, amt) in player.cards.iter() {
		player_cards_update.insert(crd, amt);
	}
	player_update.insert("cards", player_cards_update);
	update_player(&player, doc! { "$set": player_update }).await;

	// Update the deck
	let mut deck_update = Document::new();
	let mut deck_card_update = Document::new();
	for (crd, amt) in deck.cards.iter() {
		deck_card_update.insert(crd, amt);
	}
	deck_update.insert("cards", deck_card_update);
	update_deck(&deck, doc! { "$set": deck_update }).await;

	msg.reply(&ctx.http, format!("You removed **{}** from **{}**", card_str, deck.name)).await?;

	Ok(())
}

#[command("energy")]
#[aliases("e")]
#[sub_commands(deck_energy_add, deck_energy_remove)]
async fn deck_energy_main(ctx: &Context, msg: &Message) -> CommandResult {
	let content = "Here are the available deck energy commands:
	**.deck energy add <name> <type> [amount - Default: 1]** to add a basic energy to a deck.
	**.deck energy remove <name> <type> [amount - Default: 1]** to remove a basic energy from a deck.

	Energy types are:
		• ***Darkness/Dark***
		• ***Fighting***
		• ***Fire***
		• ***Grass***
		• ***Lightning/Electric***
		• ***Metal/Steel***
		• ***Psychic***
		• ***Water***
	
	For non-basic energies, those must be added via **.deck add <name> <card str>**";
	msg
		.channel_id
		.send_message(&ctx.http, |m| m.content(content))
		.await?;

	Ok(())
}

#[command("add")]
#[aliases("a")]
async fn deck_energy_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let deck_name = args.find::<String>().unwrap_or(String::from(""));
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let energy_type = args.find::<String>().unwrap_or(String::from(""));
	if energy_type == "" {
		msg.reply(&ctx.http, "You didn't provide an energy type.").await?;
		return Ok(());
	}
	let energy_card = match energy_type.to_lowercase().as_str() {
		"darkness" | "dark" => "col1-94",
		"fighting" => "col1-93",
		"fire" => "col1-89",
		"grass" => "col1-88",
		"lightning" | "electric" => "col1-91",
		"metal" | "steel" => "col1-95",
		"psychic" => "col1-92",
		"water" => "col1-90",
		_ => {
			msg.reply(&ctx.http, "You didn't select a valid basic energy type.").await?;
			return Ok(());
		},
	};
	let amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let mut deck = deck.unwrap();
	if amount + deck.cards.values().sum::<i64>() > 60 {
		msg.reply(&ctx.http, "Your deck will have more than 60 cards. Remove some to add more").await?;
		return Ok(());
	}
	*deck.cards.entry(energy_card.into()).or_insert(0) += amount;
	// Update the deck
	let mut deck_update = Document::new();
	let mut deck_card_update = Document::new();
	for (crd, amt) in deck.cards.iter() {
		deck_card_update.insert(crd, amt);
	}
	deck_update.insert("cards", deck_card_update);
	update_deck(&deck, doc! { "$set": deck_update }).await;

	msg.reply(&ctx.http, format!("You added **{} {}** energies to **{}**", amount, energy_type, deck.name)).await?;

	Ok(())
}

#[command("remove")]
#[aliases("r")]
async fn deck_energy_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let deck_name = args.find::<String>().unwrap_or(String::from(""));
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let energy_type = args.find::<String>().unwrap_or(String::from(""));
	if energy_type == "" {
		msg.reply(&ctx.http, "You didn't provide an energy type.").await?;
		return Ok(());
	}
	let energy_card = match energy_type.to_lowercase().as_str() {
		"darkness" | "dark" => "col1-94",
		"fighting" => "col1-93",
		"fire" => "col1-89",
		"grass" => "col1-88",
		"lightning" | "electric" => "col1-91",
		"metal" | "steel" => "col1-95",
		"psychic" => "col1-92",
		"water" => "col1-90",
		_ => {
			msg.reply(&ctx.http, "You didn't select a valid basic energy type.").await?;
			return Ok(());
		},
	};
	let mut amount = match args.find::<i64>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let mut deck = deck.unwrap();
	let energy_amount = deck.cards.get(energy_card.into()).unwrap_or(&0).clone();
	if energy_amount < amount {
		amount = energy_amount;
	}
	*deck.cards.entry(energy_card.into()).or_insert(0) -= amount;
	deck.cards.retain(|_, v| *v > 0);
	// Update the deck
	let mut deck_update = Document::new();
	let mut deck_card_update = Document::new();
	for (crd, amt) in deck.cards.iter() {
		deck_card_update.insert(crd, amt);
	}
	deck_update.insert("cards", deck_card_update);
	update_deck(&deck, doc! { "$set": deck_update }).await;

	msg.reply(&ctx.http, format!("You removed **{} {}** energies from **{}**", amount, energy_type, deck.name)).await?;

	Ok(())
}

#[command("display")]
#[aliases("d")]
async fn deck_display(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let deck_name = args.find::<String>().unwrap_or(String::from(""));
	if deck_name == String::from("") {
		msg.reply(&ctx.http, "You didn't provide a deck name.").await?;
		return Ok(());
	}
	let card_id = args.find::<String>().unwrap_or(String::from(""));
	if card_id == "" {
		msg.reply(&ctx.http, "You didn't provide a card.").await?;
		return Ok(());
	}
	let player = get_player(msg.author.id.0).await;
	let deck = get_deck(player.discord_id, deck_name.clone()).await;
	match deck {
		Some(_) => (),
		None => {
			msg.reply(&ctx.http, "You don't have a deck with that name.").await?;
			return Ok(());
		}
	}
	let mut deck = deck.unwrap();
	if deck.cards.contains_key(&card_id) {
		deck.display_card = String::from(&card_id);
	}
	let mut deck_update = Document::new();
	deck_update.insert("display_card", &deck.display_card);
	update_deck(&deck, doc! { "$set": deck_update }).await;

	msg.reply(&ctx.http, format!("You set **{}** display card to **{}**", deck.name, card_id)).await?;

	Ok(())
}

pub struct DeckCards {
	pub cards: Vec<(String, i64)>
}

impl DeckCards {
	pub fn from_card_str(card_str: &str) -> Self {
		let inputs = card_str.split("/").collect::<Vec<&str>>();
		let mut cards = vec![];
		for input in inputs {
			let card_amt = input
				.split(":")
				.collect::<Vec<&str>>();
			let card = String::from(card_amt[0]);
			if card_amt.len() == 1 {
				cards.push((card, 1));
			} else {
				let mut amt = card_amt[1].parse::<i64>().unwrap_or(1);
				if amt > 4 {
					amt = 4;
				}
				cards.push((card, amt));
			}
		}

		Self {
			cards
		}
	}

	pub fn player_has_all(&self, player: &Player) -> bool {
		for (card_id, amt) in &self.cards {
			if player.cards.get(card_id).unwrap_or(&0) < amt {
				return false;
			}
		}

		true
	}

	pub fn is_valid_addition(&self, deck: &Deck) -> bool {
		let deckcards_sum = self.cards
			.iter()
			.map(|ca| ca.1)
			.collect::<Vec<i64>>()
			.iter()
			.sum::<i64>();
		let deck_sum = deck.cards.values().sum::<i64>();
		if deckcards_sum + deck_sum > 60 {
			return false;
		}
		for (card_id, amt) in &self.cards {
			if deck.cards.get(card_id).unwrap_or(&0) + amt > 4 {
				return false;
			}
		}

		true
	}

	pub fn deck_has_all(&self, deck: &Deck) -> bool {
		for (card_id, amt) in &self.cards {
			if deck.cards.get(card_id).unwrap_or(&0) < amt {
				return false;
			}
		}

		true
	}
}