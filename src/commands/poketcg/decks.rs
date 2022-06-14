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
use serenity::{builder::CreateEmbed, utils::Colour};

use crate::commands::get_client;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deck {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub discord_id: i64,
	pub name: String,
	pub cards: Vec<String>
}

impl Deck {
	pub fn empty(discord_id: i64, name: String) -> Self {
		Self {
			id: None,
			discord_id,
			name,
			cards: vec![]
		}
	}

	pub fn is_valid(&self) -> bool {
		self.cards.len() == 60 as usize
	}
}

async fn get_deck_collection() -> Collection<Deck> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Deck>("decks");

	collection
}

async fn add_deck(deck: &Deck) {
	let deck_collection = get_deck_collection().await;
	deck_collection
		.insert_one(deck, None)
		.await
		.unwrap();
}

async fn get_decks_by_player(discord_id: i64) -> Vec<Deck> {
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

async fn get_deck(discord_id: i64, name: String) -> Option<Deck> {
	let deck_collection = get_deck_collection().await;
	let deck = deck_collection
		.find_one(doc! { "discord_id": discord_id, "name": name }, None)
		.await
		.unwrap();

	deck
}

async fn update_deck(deck: &Deck, update: Document) {
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