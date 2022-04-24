use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
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
static FAVOURED_WEIGHT: i32 = 30;

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

	pub fn reward(&self) -> i64 {
		match (self.slot1.as_str(), self.slot2.as_str(), self.slot3.as_str()) {
			("7", "7", "7") => 500,
			("R", "R", "R") => 200,
			("Pikachu", "Pikachu", "Pikachu") => 120,
			("Slowpoke", "Slowpoke", "Slowpoke") => 80,
			("Magnemite", "Magnemite", "Magnemite") => 50,
			("Shellder", "Shellder", "Shellder") => 30,
			("Cherry", "Cherry", "Cherry") => 15,
			("Cherry", "Cherry", _) | ("Cherry", _, "Cherry") | (_, "Cherry", "Cherry") => 5,
			_ => 0
		}
	}

	pub fn reward_display(&self) -> String {
		let slot1_id = SLOT_OPTION_IDS.get(self.slot1.as_str()).unwrap();
		let slot2_id = SLOT_OPTION_IDS.get(self.slot2.as_str()).unwrap();
		let slot3_id = SLOT_OPTION_IDS.get(self.slot3.as_str()).unwrap();
		let reward = self.reward();

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