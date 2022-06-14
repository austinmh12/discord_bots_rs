use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
		Document
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

use crate::{sets::{
	Set,
	get_sets,
	get_set
}, commands::get_client};

use super::{
	player::{
		Player,
		get_player,
		update_player
	}, Idable
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub sets: Vec<String>,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub reset: DateTime<Utc>
}

impl Store {
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
				10,
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
		let now = Utc::now() + Duration::days(1);

		Self {
			id: None,
			sets: store_sets,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	async fn update_sets(&self) -> Self {
		let sets = get_sets().await;
		let mut weighted_sets = vec![];
		for set in sets {
			let weight = set.release_date.year() - 1998;
			weighted_sets.push((set, weight));
		}
		let store_sets = weighted_sets
			.choose_multiple_weighted(
				&mut thread_rng(),
				10,
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
		let now = Utc::now() + Duration::days(1);

		Self {
			id: self.id,
			sets: store_sets,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	pub async fn embed_with_player(&self, player: Player) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		let mut desc = String::from("Welcome to the Card Store! Here you can spend cash for Packs of cards\n");
		desc.push_str(&format!("You have **${:.2}**\n", player.cash));
		desc.push_str("Here are the packs available today. To purchase packs, use **.(st)ore (b)uy <slot no. | slot id.> (amount)**\n\n");
		let discount = 1.0 + player.upgrades.store_discount as f64 * 0.05;
		for (i, set_id) in self.sets.iter().enumerate() {
			let num = i + 1;
			let set = get_set(set_id).await.unwrap();
			let (pack_type, price_mult) = if num <= 4 {
				("Pack", 1.0)
			} else if 5 <= num && num <= 7 {
				("Collection", 2.5)
			} else if 8 <= num && num <= 9 {
				("Trainer Box", 10.0)
			} else {
				("Booster Box", 30.0)
			};
			match player.completed_binders.contains(&set_id) {
				true => {
					let discount_mod = discount + 0.15;
					desc.push_str(&format!("**{} (_{}_):** {} {} - ${:.2}\n", num, set.id(), set.name, pack_type, (set.pack_price() * &price_mult) / discount_mod))
				},
				false => desc.push_str(&format!("**{} (_{}_):** {} {} - ${:.2}\n", num, set.id(), set.name, pack_type, (set.pack_price() * &price_mult) / discount))
			}
		}
		ret
			.title("Card Store")
			.description(&desc)
			.colour(Colour::from_rgb(255, 50, 20))
			.footer(|f| {
				let local_timer: DateTime<Local> = DateTime::from(self.reset);

				f.text(&format!("Resets {}", local_timer.format("%h %d %H:%M")))
			});

		ret
	}
}

async fn get_store_collection() -> Collection<Store> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Store>("store");

	collection
}

pub async fn get_store() -> Store {
	let store_collection = get_store_collection().await;
	let store = store_collection
		.find_one(None, None)
		.await
		.unwrap();
	let store = match store {
		Some(x) => x,
		None => add_store().await
	};
	if store.reset < Utc::now() {
		let store = store.update_sets().await;
		update_store(&store).await;
		return store;
	}
	

	store
}

async fn add_store() -> Store {
	let ret = Store::new().await;
	let store_collection = get_store_collection().await;
	store_collection
		.insert_one(&ret, None)
		.await
		.unwrap();
	
	ret
}

async fn update_store(store: &Store) {
	let store_collection = get_store_collection().await;
	store_collection
		.update_one(
			doc! {"_id": &store.id.unwrap() }, 
			doc! {"$set": {"sets": &store.sets, "reset": &store.reset}}, 
			None)
		.await
		.unwrap();
}

#[command("store")]
#[aliases("st")]
#[sub_commands(store_buy)]
async fn store_main(ctx: &Context, msg: &Message) -> CommandResult {
	let store = get_store().await;
	let player = get_player(msg.author.id.0).await;
	let embed = store.embed_with_player(player).await;
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
	let store_ = get_store().await;
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
	let mut player = get_player(msg.author.id.0).await;
	let (price_mult, pack_count) = if selection <= 4 {
		(1.0, 1)
	} else if 5 <= selection && selection <= 7 {
		(2.5, 4)
	} else if 8 <= selection && selection <= 9 {
		(10.0, 12)
	} else {
		(30.0, 36)
	};
	let mut discount = 1.0 + player.upgrades.store_discount as f64 * 0.05;
	if player.completed_binders.contains(&set.id()) {
		discount += 0.15;
	}
	let base_cost = (set.pack_price() * &price_mult) / discount;
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
	update_player(
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