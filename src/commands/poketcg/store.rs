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
	Datelike,
	Duration
};
use rand::{
	seq::{
		SliceRandom
	},
	prelude::*
};

use crate::sets::{
	Set,
	get_sets
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
			.map(|s| s.id.clone())
			.collect();
		let now = Utc::now() + Duration::days(1);

		Self {
			id: None,
			sets: store_sets,
			reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}
}