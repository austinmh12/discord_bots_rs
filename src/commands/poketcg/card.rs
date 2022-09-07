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
use crate::{Cache, CardCache};

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

#[async_trait]
impl Scrollable for Vec<Card> {
	async fn scroll_through(&self, ctx: &Context, msg: &Message) -> Result<(), String> {
		let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
		let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
		let save_icon = ReactionType::try_from("💾").expect("No floppy disk");
		let binder_icon = ReactionType::try_from(":pokeball:972277627077423124").expect("No pokeball");
		let mut player = player::get_player(msg.author.id.0).await;
		let embeds = self.iter().map(|e| e.embed()).collect::<Vec<_>>();
		let mut idx: i16 = 0;
		let mut content = String::from("");
		let mut message = msg
			.channel_id
			.send_message(&ctx.http, |m| {
				let mut cur_embed = embeds[idx as usize].clone();
				if embeds.len() > 1 {
					cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
				}
				let mut extra_desc = String::from("");
				if &player.current_binder.set == &self[idx as usize].set().id() {
					match player.current_binder.cards.contains(&self[idx as usize].card_id()) {
						true => extra_desc.push_str("<:pokeball:972277627077423124> In your binder\n"),
						false => extra_desc.push_str("<:GameCorner:967591653135228988> Not in your binder\n")
					}
				}
				if player.savelist.contains(&self[idx as usize].card_id()) {
					extra_desc.push_str(":white_check_mark: In your savelist");
				}
				cur_embed.description(format!("{}\n{}", &self[idx as usize].description(), extra_desc));
				m.set_embed(cur_embed);

				if embeds.len() > 1 {
					m.reactions([left_arrow.clone(), right_arrow.clone(), save_icon.clone(), binder_icon.clone()]);
				} else {
					m.reactions([save_icon.clone(), binder_icon.clone()]);
				}

				m			
			}).await.unwrap();
		
		loop {
			if embeds.len() <= 1 {
				break; // Exit before anything. Probably a way to do this before entering.
			}
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
					"💾" => {
						let card_id = &self[idx as usize].card_id();
						if player.savelist.clone().contains(&card_id) {
							let index = player.savelist.clone().iter().position(|c| c == card_id).unwrap();
							player.savelist.remove(index);
							content = format!("**{}** removed from your savelist!", &self[idx as usize].card_name());
						} else {
							player.savelist.push(card_id.clone());
							content = format!("**{}** added to your savelist!", &self[idx as usize].card_name());
						}
						player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;
					},
					"pokeball:972277627077423124" => {
						let card_id = self[idx as usize].card_id().clone();
						if player.current_binder.cards.contains(&card_id) {
							content = format!("**{}** is already in your binder!", &self[idx as usize].card_name());
						} else if &self[idx as usize].set().id() != &player.current_binder.set {
							content = String::from("This card doesn't go in your binder!");
						} else {
							let current_binder_set = sets::get_set(&player.current_binder.set).await.unwrap();
							let mut player_update = Document::new();
							*player.cards.entry(self[idx as usize].card_id()).or_insert(0) -= 1;
							if *player.cards.entry(self[idx as usize].card_id()).or_insert(0) == 0 {
								player.cards.remove(&card_id);
							}
							let mut player_cards = Document::new();
							for (crd, amt) in player.cards.iter() {
								player_cards.insert(crd, amt);
							}
							player_update.insert("cards", player_cards);
							player.current_binder.cards.push(self[idx as usize].card_id().clone());
							if player.current_binder.is_complete(ctx).await {
								player.completed_binders.push(player.current_binder.set);
								player.current_binder = binder::Binder::empty();
								player_update.insert("completed_binders", player.completed_binders.clone());
								content = format!("You completed the **{}** binder!", current_binder_set.name);
							} else {
								content = format!("You added **{}** to your binder!", &self[idx as usize].card_name());
							}
							player_update.insert("current_binder", player.current_binder.to_doc());
							let mut player_cards = Document::new();
							for (crd, amt) in player.cards.iter() {
								player_cards.insert(crd, amt);
							}
							player_update.insert("cards", player_cards);
							player::update_player(&player, doc! { "$set": player_update }).await;
						}
					}
					_ => {
						println!("{}", &emoji.as_data().as_str());
						continue
					}
				};
			} else {
				message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
				break;
			}
			message.edit(&ctx, |m| {
				let mut cur_embed = embeds[idx as usize].clone();
				if embeds.len() > 1 {
					cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
				}
				let mut extra_desc = String::from("");
				if &player.current_binder.set == &self[idx as usize].set().id() {
					match player.current_binder.cards.contains(&self[idx as usize].card_id()) {
						true => extra_desc.push_str("<:pokeball:972277627077423124> In your binder\n"),
						false => extra_desc.push_str("<:GameCorner:967591653135228988> Not in your binder\n")
					}
				}
				if player.savelist.contains(&self[idx as usize].card_id()) {
					extra_desc.push_str(":white_check_mark: In your savelist");
				}
				cur_embed.description(format!("{}\n{}", &self[idx as usize].description(), extra_desc));
				m.set_embed(cur_embed);
				m.content(content);

				m
			}).await.unwrap();

			content = String::from("");
		}

		Ok(())
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

pub async fn get_multiple_cards_by_id(ctx: &Context, card_ids: Vec<String>) -> Vec<Card> {
	let mut ret = vec![];
	let cached_cards = get_multiple_cards_from_cache(ctx, &card_ids).await;
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
		let chunk_cards = get_cards_with_query(ctx, &format!("({})", inner_query)).await;
		ret.extend(chunk_cards);
	}
	// If we've gotten here there are cards to cache
	add_cards(ctx, ret.clone()).await;
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
		let chunk_cards = get_cards_with_query_without_cache_add(&format!("({})", inner_query)).await;
		ret.extend(chunk_cards.iter().map(|c| (c.id(), c.clone())));
	}

	ret
}

pub async fn get_card(ctx: &Context, id: &str) -> Card {
	let cached_card = get_card_from_cache(ctx, id).await;
	match cached_card {
		Some(c) => c,
		None => {
			let data = api_call(&format!("cards/{}", id), None)
				.await
				.unwrap();
			let card_data = &data["data"];
			let card = Card::from_json(&card_data);
			add_card(ctx, card.clone()).await;
		
			card
		}
	}
}

pub async fn get_cards_with_query(ctx: &Context, query: &str) -> Vec<Card> {
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
	add_cards(ctx, ret.clone()).await;

	ret
}

pub async fn get_cards_with_query_without_cache_add(query: &str) -> Vec<Card> {
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

	ret
}


pub async fn get_cards_by_set(ctx: &Context, set: &Set) -> Vec<Card> {
	let mut ret = vec![];
	let cached_cards = get_cards_from_cache_by_set(ctx, set).await;
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
	add_cards(ctx, ret.clone()).await;
	ret.extend(cached_cards);

	ret
}

async fn get_card_collection() -> Collection<Card> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Card>("cards");

	collection
}

// async fn add_card(card: &Card) {
// 	let card_collection = get_card_collection().await;
// 	card_collection
// 		.insert_one(card, None)
// 		.await
// 		.unwrap();
// }

async fn add_card(ctx: &Context, card: Card) {
	let card_cache = CardCache::new(card.clone());
	let cache_lock = {
		let cache_read = ctx.data.read().await;
		
		cache_read.get::<Cache>().expect("Expected a Cache in TypeMap").clone()
	};
	{
		let mut cache = cache_lock.write().await;
		cache.insert(card.card_id, card_cache);
	}
}

// async fn add_cards(cards: &Vec<Card>) {
// 	if cards.len() <= 0 {
// 		return;
// 	}
// 	let cached_cards = get_cards_from_cache().await;
// 	let mut new_cards = vec![];
// 	for card in cards {
// 		if !cached_cards.contains(card) {
// 			new_cards.push(card);
// 		}
// 	}
// 	if new_cards.len() <= 0 {
// 		return;
// 	}
// 	let card_collection = get_card_collection().await;
// 	card_collection
// 		.insert_many(new_cards, None)
// 		.await
// 		.unwrap();
// }

async fn add_cards(ctx: &Context, cards: Vec<Card>) {
	let card_caches = cards
		.iter()
		.map(|c| (c.clone(), CardCache::new(c.clone())))
		.collect::<Vec<(Card, CardCache)>>();
	let cache_lock = {
		let cache_read = ctx.data.read().await;
		
		cache_read.get::<Cache>().expect("Expected a Cache in TypeMap").clone()
	};
	{
		let mut cache = cache_lock.write().await;
		for (card, card_cache) in card_caches {
			cache.insert(card.card_id, card_cache);
		}
	}
}

// async fn get_card_from_cache(id: &str) -> Option<Card> {
// 	let card_collection = get_card_collection().await;
// 	let card = card_collection
// 		.find_one(doc! { "card_id": id }, None)
// 		.await
// 		.unwrap();

// 	card
// }

async fn get_card_from_cache(ctx: &Context, id: &str) -> Option<Card> {
	let cached_card = {
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		cache.get(id).map_or(None, |x| Some(x.clone()))
	};
	let ret = match cached_card {
		Some(x) => Some(x.card),
		None => None
	};

	ret
}

// async fn get_cards_from_cache() -> Vec<Card> {
// 	let card_collection = get_card_collection().await;
// 	let cards = card_collection
// 		.find(None, None)
// 		.await
// 		.unwrap()
// 		.try_collect::<Vec<Card>>()
// 		.await
// 		.unwrap();

// 	cards
// }

async fn get_cards_from_cache(ctx: &Context) -> Vec<Card> {
	let cards = {
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		cache
			.values()
			.into_iter()
			.map(|cc| cc.card.clone())
			.collect::<Vec<Card>>()
	};

	cards
}

// async fn get_multiple_cards_from_cache(card_ids: &Vec<String>) -> Vec<Card> {
// 	if card_ids.len() == 0 {
// 		return vec![];
// 	}
// 	let card_collection = get_card_collection().await;
// 	let mut docs = vec![];
// 	for card_id in card_ids {
// 		docs.push(doc!{"card_id": card_id});
// 	}
// 	let cards = card_collection
// 		.find(doc! { "$or": docs }, None)
// 		.await
// 		.unwrap()
// 		.try_collect::<Vec<Card>>()
// 		.await
// 		.unwrap();

// 	cards
// }

async fn get_multiple_cards_from_cache(ctx: &Context, card_ids: &Vec<String>) -> Vec<Card> {
	if card_ids.len() == 0 {
		return vec![];
	}
	let cards = {
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		cache
			.iter()
			.filter(|(cid, _)| card_ids.contains(cid))
			.map(|(_, cc)| cc.card.clone())
			.collect::<Vec<Card>>()
	};

	cards
}

// async fn get_cards_from_cache_by_set(set: &Set) -> Vec<Card> {
// 	let card_collection = get_card_collection().await;
// 	let cards = card_collection
// 		.find(doc!{"set.set_id": set.id()}, None)
// 		.await
// 		.unwrap()
// 		.try_collect::<Vec<Card>>()
// 		.await
// 		.unwrap();

// 	cards
// }

async fn get_cards_from_cache_by_set(ctx: &Context, set: &Set) -> Vec<Card> {
	let cards = {
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		cache
			.iter()
			.filter(|(_, cc)| cc.card.set().id() == set.id())
			.map(|(_, cc)| cc.card.clone())
			.collect::<Vec<Card>>()
	};

	cards
}

// pub async fn get_outdated_cards() -> Vec<Card> {
// 	let card_collection = get_card_collection().await;
// 	let cards = card_collection
// 		.find(doc!{"last_check": {"$lt": Utc::now()}}, None)
// 		.await
// 		.unwrap()
// 		.try_collect::<Vec<Card>>()
// 		.await
// 		.unwrap();

// 	cards
// }

pub async fn get_outdated_cards(ctx: &Context) -> Vec<CardCache> {
	let cards = {
		let cache_read = ctx.data.read().await;
		let cache_lock = cache_read.get::<Cache>().expect("Expected Cache in TypeMap").clone();
		let cache = cache_lock.read().await;

		cache
			.iter()
			.filter(|(_, cc)| cc.last_updated < Utc::now())
			.map(|(_, cc)| cc.clone())
			.collect::<Vec<CardCache>>()
	};

	cards
}

// pub async fn update_cached_cards(cards: Vec<Card>) {
// 	let card_collection = get_card_collection().await;
// 	let mut threads = vec![];
// 	for card in cards {
// 		let card_col = card_collection.clone();
// 		threads.push(task::spawn(async move {
// 			card_col.update_one(
// 				doc! {"_id": card.id},
// 				doc! {"$set": { "price": card.price, "last_check": card.last_check }}, 
// 				None
// 			)
// 				.await
// 				.unwrap();
// 		}))
// 	}
// }

pub async fn update_cached_cards(ctx: &Context, cards: Vec<CardCache>) {
	let cache_lock = {
		let cache_read = ctx.data.read().await;
		
		cache_read.get::<Cache>().expect("Expected a Cache in TypeMap").clone()
	};
	{
		let mut cache = cache_lock.write().await;
		for card_cache in cards {
			cache
				.entry(card_cache.clone().card.card_id)
				.or_insert(card_cache).last_updated = Utc::now() + Duration::days(1);
		}
		cache.retain(|cid, cc| cc.last_accessed > Utc::now() - Duration::days(3));
	}
}

#[command("card")]
async fn search_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let _player = player::get_player(msg.author.id.0).await;
	let search_str = args.rest();
	let cards = get_cards_with_query(ctx, &format!("{}", search_str))
		.await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "No cards found.").await?;
	} else {
		cards.scroll_through(ctx, msg).await?;
	}

	Ok(())
}