use std::collections::HashMap;

use super::{
	*,
};
use futures::TryStreamExt;
use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
	}, 
	Collection
};
use serde::{Serialize, Deserialize};
use chrono::{
	DateTime, 
	Utc,
};
use tokio::task;
use crate::{sets::Set, commands::get_client};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub card_id: String,
	pub name: String,
	pub set: Set,
	pub number: String,
	pub price: f64,
	pub image: String,
	pub rarity: String,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub last_check: DateTime<Utc>
}

impl Card {
	pub fn from_json(obj: &serde_json::Value) -> Self {
		let price = match obj.pointer("/tcgplayer/prices/normal/market") {
			Some(x) => x.as_f64().unwrap(),
			None => match obj.pointer("/tcgplayer/prices/normal/mid") {
				Some(y) => y.as_f64().unwrap(),
				None => match obj.pointer("/tcgplayer/prices/holofoil/market") {
					Some(z) => z.as_f64().unwrap(),
					None => match obj.pointer("/tcgplayer/prices/holofoil/mid") {
						Some(t) => t.as_f64().unwrap(),
						None => match obj.pointer("/tcgplayer/prices/reverseHolofoil/market") {
							Some(w) => w.as_f64().unwrap(),
							None => match obj.pointer("/tcgplayer/prices/reverseHolofoil/mid") {
								Some(a) => a.as_f64().unwrap(),
								None => match obj.pointer("/tcgplayer/prices/1stEditionNormal/market") {
									Some(b) => b.as_f64().unwrap(),
									None => match obj.pointer("/cardmarket/prices/averageSellPrice") {
										Some(c) => c.as_f64().unwrap(),
										None => 0.01
									}
								}
							}
						}
					}
				}
			}
		};
		let rarity = match obj.get("rarity") {
			Some(x) => String::from(x.as_str().unwrap()),
			None => String::from("Unknown")
		};

		Self {
			id: None,
			card_id: String::from(obj["id"].as_str().unwrap()),
			name: String::from(obj["name"].as_str().unwrap()),
			set: Set::from_json(obj.get("set").unwrap()),
			number: String::from(obj["number"].as_str().unwrap()),
			price: price,
			image: String::from(obj["images"]["large"].as_str().unwrap()),
			rarity,
			last_check: Utc::now()
		}
	}
}

impl PaginateEmbed for Card {
	fn embed(&self) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		ret
			.title(&self.name)
			.description(&self.description())
			.colour(Colour::from_rgb(255, 50, 20))
			.image(&self.image);

		ret
	}
}

impl CardInfo for Card {
	fn card_id(&self) -> String {
		self.card_id.clone()
	}

	fn card_name(&self) -> String {
		self.name.clone()
	}

	fn description(&self) -> String {
		format!("**ID:** {}\n**Rarity:** {}\n**Price:** ${:.2}", &self.card_id, &self.rarity, &self.price)
	}

	fn price(&self) -> f64 {
		self.price.clone()
	}
}

impl Idable for Card {
	fn id(&self) -> String {
		self.card_id.clone()
	}
}

impl HasSet for Card {
	fn set(&self) -> sets::Set {
		self.set.clone()
	}
}

impl PartialEq for Card {
	fn eq(&self, other: &Self) -> bool {
		self.card_id == other.card_id
	}

	fn ne(&self, other: &Self) -> bool {
		self.card_id != other.card_id
	}
}

// pub async fn get_cards() -> Vec<Card> {
// 	let mut ret = <Vec<Card>>::new();
// 	let data = api_call("cards", None).await.unwrap();
// 	let card_data = data["data"].as_array().unwrap();
// 	for cd in card_data {
// 		let card = Card::from_json(cd);
// 		ret.push(card);
// 	}

// 	ret
// }

pub async fn get_multiple_cards_by_id(card_ids: Vec<String>) -> Vec<Card> {
	let mut ret = vec![];
	let cached_cards = get_multiple_cards_from_cache(&card_ids).await;
	if cached_cards.len() == card_ids.len() {
		return cached_cards;
	}
	let card_id_chunks: Vec<Vec<String>> = card_ids.chunks(250).map(|x| x.to_vec()).collect();
	for card_id_chunk in card_id_chunks {
		let inner_query = card_id_chunk
			.iter()
			.map(|c| format!("id:{}", c))
			.collect::<Vec<String>>()
			.join(" OR ");
		let chunk_cards = get_cards_with_query(&format!("({})", inner_query)).await;
		ret.extend(chunk_cards);
	}
	// If we've gotten here there are cards to cache
	add_cards(&ret).await;
	ret.extend(cached_cards);

	ret
}

pub async fn get_multiple_cards_by_id_without_cache(card_ids: Vec<String>) -> HashMap<String, Card> {
	let mut ret = HashMap::new();
	let card_id_chunks: Vec<Vec<String>> = card_ids.chunks(250).map(|x| x.to_vec()).collect();
	for card_id_chunk in card_id_chunks {
		let inner_query = card_id_chunk
			.iter()
			.map(|c| format!("id:{}", c))
			.collect::<Vec<String>>()
			.join(" OR ");
		let chunk_cards = get_cards_with_query(&format!("({})", inner_query)).await;
		ret.extend(chunk_cards.iter().map(|c| (c.id(), c.clone())));
	}

	ret
}

pub async fn get_card(id: &str) -> Card {
	let cached_card = get_card_from_cache(id).await;
	match cached_card {
		Some(c) => c,
		None => {
			let data = api_call(&format!("cards/{}", id), None)
				.await
				.unwrap();
			let card_data = &data["data"];
			let card = Card::from_json(&card_data);
			add_card(&card).await;
		
			card
		}
	}
}

pub async fn get_cards_with_query(query: &str) -> Vec<Card> {
	let mut ret = <Vec<Card>>::new();
	let mut data = api_call("cards", Some(vec![("q", query)])).await.unwrap();
	let card_data = data["data"].as_array().unwrap();
	for cd in card_data {
		let card = Card::from_json(cd);
		ret.push(card);
	}
	let mut page = 1;
	while data["count"].as_i64().unwrap() > 0 {
		page += 1;
		data = api_call("cards", Some(vec![("q", query), ("page", page.to_string().as_str())])).await.unwrap();
		let card_data = data["data"].as_array().unwrap();
		for cd in card_data {
			let card = Card::from_json(cd);
			ret.push(card);
		}
	}
	add_cards(&ret).await;

	ret
}

pub async fn get_cards_by_set(set: &Set) -> Vec<Card> {
	let mut ret = vec![];
	let cached_cards = get_cards_from_cache_by_set(set).await;
	if cached_cards.len() >= set.total as usize {
		return cached_cards;
	}
	let mut data = api_call("cards", Some(vec![("q", &format!("set.id:{}", set.id()))])).await.unwrap();
	let card_data = data["data"].as_array().unwrap();
	for cd in card_data {
		let card = Card::from_json(cd);
		if !cached_cards.contains(&card) {
			ret.push(card);
		}
	}
	let mut page = 1;
	while data["count"].as_i64().unwrap() > 0 {
		page += 1;
		data = api_call("cards", Some(vec![("q", &format!("set.id:{}", set.id())), ("page", page.to_string().as_str())])).await.unwrap();
		let card_data = data["data"].as_array().unwrap();
		for cd in card_data {
			let card = Card::from_json(cd);
			if !cached_cards.contains(&card) {
				ret.push(card);
			}
		}
	}
	// If we've gotten here there are cards to cache
	add_cards(&ret).await;
	ret.extend(cached_cards);

	ret
}

async fn get_card_collection() -> Collection<Card> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Card>("cards");

	collection
}

async fn add_card(card: &Card) {
	let card_collection = get_card_collection().await;
	card_collection
		.insert_one(card, None)
		.await
		.unwrap();
}

async fn add_cards(cards: &Vec<Card>) {
	if cards.len() <= 0 {
		return;
	}
	let cached_cards = get_cards_from_cache().await;
	let mut new_cards = vec![];
	for card in cards {
		if !cached_cards.contains(card) {
			new_cards.push(card);
		}
	}
	if new_cards.len() <= 0 {
		return;
	}
	let card_collection = get_card_collection().await;
	card_collection
		.insert_many(new_cards, None)
		.await
		.unwrap();
}

async fn get_card_from_cache(id: &str) -> Option<Card> {
	let card_collection = get_card_collection().await;
	let card = card_collection
		.find_one(doc! { "card_id": id }, None)
		.await
		.unwrap();

	card
}

async fn get_cards_from_cache() -> Vec<Card> {
	let card_collection = get_card_collection().await;
	let cards = card_collection
		.find(None, None)
		.await
		.unwrap()
		.try_collect::<Vec<Card>>()
		.await
		.unwrap();

	cards
}

async fn get_multiple_cards_from_cache(card_ids: &Vec<String>) -> Vec<Card> {
	if card_ids.len() == 0 {
		return vec![];
	}
	let card_collection = get_card_collection().await;
	let mut docs = vec![];
	for card_id in card_ids {
		docs.push(doc!{"card_id": card_id});
	}
	let cards = card_collection
		.find(doc! { "$or": docs }, None)
		.await
		.unwrap()
		.try_collect::<Vec<Card>>()
		.await
		.unwrap();

	cards
}

async fn get_cards_from_cache_by_set(set: &Set) -> Vec<Card> {
	let card_collection = get_card_collection().await;
	let cards = card_collection
		.find(doc!{"set.set_id": set.id()}, None)
		.await
		.unwrap()
		.try_collect::<Vec<Card>>()
		.await
		.unwrap();

	cards
}

pub async fn get_outdated_cards() -> Vec<Card> {
	let card_collection = get_card_collection().await;
	let cards = card_collection
		.find(doc!{"last_check": {"$lt": Utc::now()}}, None)
		.await
		.unwrap()
		.try_collect::<Vec<Card>>()
		.await
		.unwrap();

	cards
}

pub async fn update_cached_cards(cards: Vec<Card>) {
	let card_collection = get_card_collection().await;
	let mut threads = vec![];
	for card in cards {
		let card_col = card_collection.clone();
		threads.push(task::spawn(async move {
			card_col.update_one(
				doc! {"_id": card.id},
				doc! {"$set": { "price": card.price, "last_check": card.last_check }}, 
				None
			)
				.await
				.unwrap();
		}))
	}
}

#[command("card")]
async fn search_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let player = player::get_player(msg.author.id.0).await;
	let search_str = args.rest();
	let cards = get_cards_with_query(&format!("{}", search_str))
		.await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "No cards found.").await?;
	} else {
		card_paginated_embeds(ctx, msg, cards, player).await?;
	}

	Ok(())
}