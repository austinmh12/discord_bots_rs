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

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
	id: Option<ObjectId>,
	pub sets: Vec<String>,
	pub reset: DateTime<Utc>
}