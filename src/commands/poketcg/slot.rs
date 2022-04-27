use std::collections::HashMap;
use serenity::{builder::CreateEmbed, utils::Colour};
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
use crate::{
	sets::{
		Set,
		get_sets,
		get_set
	},
	card::{
		get_cards_with_query,
		get_card,
	},
	player::Player,
	commands::get_client
};

use super::Idable;

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

impl TokenShop {
	async fn new() -> Self {
		let sets = get_sets().await;
		let mut weighted_sets = vec![];
		for set in sets {
			let weight = set.release_date.year() - 1998;
			weighted_sets.push((set, weight));
		}
		let store_sets = weighted_sets
			.choose_multiple_weighted(
				&mut thread_rng(),
				3,
				|ws| ws.1
			)
			.unwrap()
			.collect::<Vec<_>>()
			.iter()
			.map(|ws| ws.0.clone())
			.collect::<Vec<Set>>()
			.iter()
			.map(|s| s.id())
			.collect();
		let letters = "abcdefghijklmnopqrstuvwxyz";
		let rand_letter_start = letters.chars().choose(&mut thread_rng()).unwrap();
		let rare_cards_no_rainbow = get_cards_with_query(&format!("name:{}* AND -rarity:Common AND -rarity:Uncommon AND -rarity:*Rainbow", rand_letter_start))
			.await;
		let rainbows = get_cards_with_query("rarity:*Rainbow")
			.await;
		let rare_card = rare_cards_no_rainbow
			.iter()
			.choose(&mut thread_rng())
			.unwrap()
			.clone()
			.id();
		let rainbow_card = rainbows
			.iter()
			.choose(&mut thread_rng())
			.unwrap()
			.clone()
			.id();
		let now = Utc::now() + Duration::days(1);

		Self {
			id: None,
			sets: store_sets,
			rare_card,
			rainbow_card,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	async fn update_shop(&self) -> Self {
		let tmp_tokenshop = TokenShop::new().await;
		let now = Utc::now() + Duration::days(1);

		Self {
			id: self.id,
			sets: tmp_tokenshop.sets,
			rare_card: tmp_tokenshop.rare_card,
			rainbow_card: tmp_tokenshop.rainbow_card,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	pub async fn embed_with_player(&self, player: Player) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		let mut desc = String::from("Welcome to the **Token Shop**! Here you can spend tokens for prized\n");
		desc.push_str(&format!("You have **{}** tokens\n", player.tokens));
		desc.push_str("Here are the prizes available today. To purchase one, use **.gamecorner tokenstore (b)uy <slot no> [amount - Default 1]**\n\n");
		for (i, set_id) in self.sets.iter().enumerate() {
			let num = i + 1;
			let set = get_set(set_id).await.unwrap();
			desc.push_str(&format!("**{}:** {} - {} tokens\n", num, set.name, to_tokens(set.pack_price())));
		}
		let rare_card = get_card(&self.rare_card).await;
		desc.push_str(&format!("**4:** {} (_{}_) - {} tokens\n", rare_card.name, rare_card.id(), to_tokens(rare_card.price) * 10));
		let rainbow_card = get_card(&self.rainbow_card).await;
		desc.push_str(&format!("**5:** {} (_{}_) - {} tokens", rainbow_card.name, rainbow_card.id(), to_tokens(rainbow_card.price) * 10));
		ret
			.description(&desc)
			.colour(Colour::from_rgb(255, 50, 20))
			.footer(|f| {
				let local_timer: DateTime<Local> = DateTime::from(self.reset);

				f.text(&format!("Resets {}", local_timer.format("%h %d %H:%m")))
			})
			.author(|a| a
				.icon_url("https://archives.bulbagarden.net/media/upload/9/92/Bag_Coin_Case_Sprite.png")
				.name("Token Shop")
			);

		ret
	}
}

pub fn to_tokens(price: f64) -> i64 {
	let inflation = price * 1.25 * 10.0;

	inflation as i64
}

async fn get_token_shop_collection() -> Collection<TokenShop> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<TokenShop>("tokenshop");

	collection
}

pub async fn get_token_shop() -> TokenShop {
	let token_shop_collection = get_token_shop_collection().await;
	let token_shop = token_shop_collection
		.find_one(None, None)
		.await
		.unwrap();
	let token_shop = match token_shop {
		Some(x) => x,
		None => add_token_shop().await
	};
	if token_shop.reset < Utc::now() {
		let token_shop = token_shop.update_shop().await;
		update_token_shop(&token_shop).await;
		return token_shop;
	}
	
	token_shop
}

async fn add_token_shop() -> TokenShop {
	let ret = TokenShop::new().await;
	let token_shop_collection = get_token_shop_collection().await;
	token_shop_collection
		.insert_one(&ret, None)
		.await
		.unwrap();
	
	ret
}

async fn update_token_shop(token_shop: &TokenShop) {
	let token_shop_collection = get_token_shop_collection().await;
	token_shop_collection
		.update_one(
			doc! {"_id": &token_shop.id.unwrap() }, 
			doc! {"$set": {
				"sets": &token_shop.sets,
				"rare_card": &token_shop.rare_card,
				"rainbow_card": &token_shop.rainbow_card,
				"reset": &token_shop.reset
			}},
			None)
		.await
		.unwrap();
}