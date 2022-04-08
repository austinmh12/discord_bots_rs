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
	TimeZone,
	DateTime, 
	Utc,
};
use futures::stream::{TryStreamExt};

use crate::{
	commands::get_client
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub discord_id: u64,
	pub cash: u64,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub daily_reset: DateTime<Utc>, // Need to learn to work with datetimes
	pub packs: Vec<(String, u32)>,
	pub packs_opened: u64,
	pub packs_bought: u64,
	pub total_cash: u64,
	pub total_cards: u64,
	pub cards_sold: u64,
	pub daily_packs: u16,
	pub quiz_questions: u16,
	pub current_multiplier: u16,
	pub quiz_correct: u64,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub quiz_reset: DateTime<Utc>, // Need to learn to work with datetimes
	pub savelist: Vec<String>,
	pub perm_multiplier: u64
}

impl Player {
	fn new_from_discord_id(discord_id: u64) -> Self {
		Self {
			id: None,
			discord_id,
			cash: 25,
			daily_reset: Utc::now(),
			packs: vec![],
			packs_opened: 0,
			packs_bought: 0,
			total_cash: 25,
			total_cards: 0,
			cards_sold: 0,
			daily_packs: 50,
			quiz_questions: 5,
			current_multiplier: 1,
			quiz_correct: 0,
			quiz_reset: Utc::now(),
			savelist: vec![],
			perm_multiplier: 0
		}
	}
}

async fn get_player_collection() -> Collection<Player> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Player>("players");

	collection
}

// Database functions
async fn get_players() -> Vec<Player> { // Will change to Player
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

async fn get_player(discord_id: u64) -> Player { // Will change to Player
	let player_collection = get_player_collection().await;
	let player = player_collection
		.find_one(doc! { "discord_id": discord_id as i64 }, None)
		.await
		.unwrap();
	match player {
		Some(x) => return x,
		None => return add_player(discord_id)
	}
}

fn add_player(discord_id: u64) -> Player {
	let ret = Player::new_from_discord_id(discord_id);
	
	ret
}

// pub async fn add_player(discord_id: u64) -> Player { // Will change to Player
// 	let ret = Player::new_from_discord_id(discord_id);
// 	let database = get_database_connection().await;
// 	sqlx::query!(
// 		"insert into players values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
// 		ret.discord_id,
// 		ret.cash,
// 		ret.daily_reset,
// 		ret.packs
// 			.iter()
// 			.map(|p| p.set.id)
// 			.collect::<Vec<String>>()
// 			.join(","),
// 		ret.packs_opened,
// 		ret.packs_bought,
// 		ret.total_cash,
// 		ret.total_cards,
// 		ret.cards_sold,
// 		ret.daily_packs,
// 		ret.quiz_questions,
// 		ret.current_multiplier,
// 		ret.quiz_correct,
// 		ret.quiz_reset,
// 		ret.savelist
// 			.join(","),
// 		ret.perm_multiplier
// 	)
// 		.execute(&database)
// 		.await
// 		.unwrap();

// 	ret
// }