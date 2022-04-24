use chrono::{
	DateTime, 
	Duration,
	Utc,
	TimeZone,
	Datelike,
};
use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		oid::ObjectId
	},
	Collection
};

use crate::commands::get_client;

fn utc_now() -> DateTime<Utc> {
	let now = Utc::now() + Duration::days(1);

	Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timer {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub pack_reset: DateTime<Utc>,
	#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime", default = "utc_now")]
	pub slot_reset: DateTime<Utc>
}

impl Timer {
	fn new() -> Self {
		let now = Utc::now() + Duration::days(1);

		Self {
			id: None,
			pack_reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0),
			slot_reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}

	fn update_timers(&self) -> Self {
		let now = Utc::now() + Duration::days(1);

		Self {
			id: self.id,
			pack_reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0),
			slot_reset: Utc.ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0)
		}
	}
}

async fn get_timer_collection() -> Collection<Timer> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<Timer>("timers");

	collection
}

pub async fn get_timer() -> Timer {
	let timer_collection = get_timer_collection().await;
	let timer = timer_collection
		.find_one(None, None)
		.await
		.unwrap();
	let timer = match timer {
		Some(x) => x,
		None => add_timer().await
	};
	
	timer
}

async fn add_timer() -> Timer {
	let ret = Timer::new();
	let timer_collection = get_timer_collection().await;
	timer_collection
		.insert_one(&ret, None)
		.await
		.unwrap();

	ret
}

pub async fn update_timer(timer: &Timer) {
	let timer = timer.update_timers();
	let timer_collection = get_timer_collection().await;
	timer_collection
		.update_one(
			doc! { "_id": &timer.id.unwrap() }, 
			doc! {"$set": {"pack_reset": &timer.pack_reset, "slot_reset": &timer.slot_reset}}, 
			None)
		.await
		.unwrap();
}