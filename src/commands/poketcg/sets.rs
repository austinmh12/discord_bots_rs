use crate::commands::get_client;

use super::*;
use chrono::{
	NaiveDate,
	DateTime,
	Utc, 
	Datelike,
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
pub struct Set {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub set_id: String,
	pub name: String,
	pub series: String,
	pub printed: i32,
	pub total: i32,
	pub logo: String,
	pub symbol: String,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub release_date: DateTime<Utc>
}

impl Set {
	pub fn from_json(obj: &serde_json::Value) -> Self {
		Self {
			id: None,
			set_id: String::from(obj["id"].as_str().unwrap()),
			name: String::from(obj["name"].as_str().unwrap()),
			series: String::from(obj["series"].as_str().unwrap()),
			printed: obj["printedTotal"].as_i64().unwrap() as i32,
			total: obj["total"].as_i64().unwrap() as i32,
			logo: String::from(obj["images"]["logo"].as_str().unwrap()),
			symbol: String::from(obj["images"]["symbol"].as_str().unwrap()),
			release_date: DateTime::<Utc>::from_utc(
				NaiveDate::parse_from_str(obj["releaseDate"].as_str().unwrap(), "%Y/%m/%d")
					.unwrap()
					.and_hms(0, 0, 0),
				Utc
			)
		}
	}
	
	pub fn pack_price(&self) -> f64 {
		let now = Utc::now();
		let date_power = now.year() - &self.release_date.year();

		((3.75 * 1.1_f64.powi(date_power)) * 100.0).round() / 100.0
	}

	pub fn description(&self) -> String {
		format!("**ID:** {}\n**Series:** {}\n**Total cards:** {}\n**Pack price:** ${:.2}", &self.set_id, &self.series, &self.printed, &self.pack_price())
	}
}

impl PaginateEmbed for Set {
	fn embed(&self) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		ret
			.title(&self.name)
			.description(&self.description())
			.colour(Colour::from_rgb(255, 50, 20))
			.image(&self.logo)
			.thumbnail(&self.symbol);

		ret
	}
}

impl Idable for Set {
	fn id(&self) -> String {
		self.set_id.clone()
	}
}

pub async fn get_sets() -> Vec<Set> {
	let mut ret = vec![];
	let cached_sets = get_sets_from_cache().await;
	let inner_query = cached_sets
		.iter()
		.map(|s| format!("-id:{}", s.id()))
		.collect::<Vec<String>>()
		.join(" AND ");
	let data = api_call("sets", Some(vec![("q", &format!("({})", inner_query))]))
		.await
		.unwrap();
	let set_data = data["data"].as_array().unwrap();
	for s in set_data {
		let set = Set::from_json(s);
		ret.push(set);
	}
	// At this point, ret is only sets that haven't been cached, so add them
	add_sets(&ret).await;
	ret.extend(cached_sets);

	ret
}

pub async fn get_set(id: &str) -> Option<Set> {
	let cached_set = get_set_from_cache(id).await;
	match cached_set {
		Some(s) => Some(s),
		None => {
			let data = api_call(&format!("sets/{}", id), None)
				.await
				.unwrap();
			let set_data = data.get("data");
			let set = match set_data {
				Some(x) => {
					let s = Set::from_json(x);
					add_set(&s).await;
					
					Some(s)
				},
				None => None
			};
		
			set
		}
	}
}

pub async fn get_sets_with_query(query: &str) -> Vec<Set> {
	let mut ret = vec![];
	let data = api_call("sets", Some(vec![("q", query)])).await.unwrap();
	let set_data = data["data"].as_array().unwrap();
	for st in set_data {
		let set = Set::from_json(st);
		ret.push(set);
	}

	ret
}

async fn get_set_collection() -> Collection<Set> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Set>("sets");

	collection
}

async fn add_set(set: &Set) {
	let set_collection = get_set_collection().await;
	set_collection
		.insert_one(set, None)
		.await
		.unwrap();
}

async fn add_sets(sets: &Vec<Set>) {
	if sets.len() <= 0 {
		return;
	}
	let set_collection = get_set_collection().await;
	set_collection
		.insert_many(sets, None)
		.await
		.unwrap();
}

async fn get_set_from_cache(id: &str) -> Option<Set> {
	let set_collection = get_set_collection().await;
	let set = set_collection
		.find_one(doc! { "set_id": id }, None)
		.await
		.unwrap();

	set
}

async fn get_sets_from_cache() -> Vec<Set> {
	let set_collection = get_set_collection().await;
	let sets = set_collection
		.find(None, None)
		.await
		.unwrap()
		.try_collect::<Vec<Set>>()
		.await
		.unwrap();

	sets
}

#[command("set")]
async fn search_set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let search_str = args.rest();
	let sets = get_sets_with_query(&format!("{}", search_str))
		.await;
	if sets.len() == 0 {
		msg.reply(&ctx.http, "No sets found.").await?;
	} else {
		set_paginated_embeds(ctx, msg, sets).await?;
	}

	Ok(())
}

#[command("sets")]
async fn sets_command(ctx: &Context, msg: &Message) -> CommandResult {
	let sets = get_sets().await;
	set_paginated_embeds(ctx, msg, sets).await?;

	Ok(())
}

#[command("set")]
async fn set_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let set_id = args.rest();
	let set = get_set(set_id).await;
	match set {
		Some(x) => set_paginated_embeds(ctx, msg, vec![x]).await?,
		None => {
			msg.reply(&ctx.http, "No set found with that id.").await?;
		}
	}

	Ok(())
}