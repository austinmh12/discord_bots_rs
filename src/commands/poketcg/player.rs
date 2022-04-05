use sqlx::{
	Sqlite, 
	sqlite::{
		SqliteTypeInfo,
		type_info::{
			DataType
		}
	}
};

use crate::{
	packs::Pack,
	commands::get_database_connection,
};

impl Type<Sqlite> for u64 {
	fn type_info() -> SqliteTypeInfo {
		SqliteTypeInfo(DataType::Int64)
	}

	fn compatible(ty: &SqliteTypeInfo) -> bool {
		matches!(ty.0, DataType::Int | DataType::Int64)
	}
}

pub struct Player {
	pub discord_id: u64,
	pub cash: u64,
	pub daily_reset: i64, // Need to learn to work with datetimes
	pub packs: Vec<Pack>,
	pub packs_opened: u64,
	pub packs_bought: u64,
	pub total_cash: u64,
	pub total_cards: u64,
	pub cards_sold: u64,
	pub daily_packs: u16,
	pub quiz_questions: u16,
	pub current_multiplier: u16,
	pub quiz_correct: u64,
	pub quiz_reset: i64, // Need to learn to work with datetimes
	pub savelist: Vec<String>,
	pub perm_multiplier: u64
}

impl Player {
	fn new_from_discord_id(discord_id: u64) -> Self {
		Self {
			discord_id,
			cash: 25,
			daily_reset: 0,
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
			quiz_reset: 0,
			savelist: vec![],
			perm_multiplier: 0
		}
	}
}



// Database functions
async fn get_players() -> Vec<Player> { // Will change to Player
	let ret = vec![];

	ret
}

async fn get_player(discord_id: u64) -> String { // Will change to Player
	let ret = String::from("");

	ret
}

pub async fn add_player(discord_id: u64) -> Player { // Will change to Player
	let ret = Player::new_from_discord_id(discord_id);
	let database = get_database_connection().await;
	sqlx::query!(
		"insert into players values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
		ret.discord_id,
		ret.cash,
		ret.daily_reset,
		ret.packs
			.iter()
			.map(|p| p.set.id)
			.collect::<Vec<String>>()
			.join(","),
		ret.packs_opened,
		ret.packs_bought,
		ret.total_cash,
		ret.total_cards,
		ret.cards_sold,
		ret.daily_packs,
		ret.quiz_questions,
		ret.current_multiplier,
		ret.quiz_correct,
		ret.quiz_reset,
		ret.savelist
			.join(","),
		ret.perm_multiplier
	)
		.execute(&database)
		.await
		.unwrap();

	ret
}