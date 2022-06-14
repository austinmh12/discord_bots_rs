use serde::{Serialize, Deserialize};
use mongodb::{
	bson::{
		doc,
		Document,
		oid::ObjectId,
	}, 
	Collection
};
use serenity::{builder::CreateEmbed, utils::Colour};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deck {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub discord_id: i64,
	pub name: String,
	pub cards: Vec<String>
}

impl Deck {
	pub fn empty(discord_id: i64, name: String) -> Self {
		Self {
			id: None,
			discord_id,
			name,
			cards: vec![]
		}
	}

	pub fn to_doc(&self) -> Document {
		let mut d = Document::new();
		d.insert("id", &self.id);
		d.insert("discord_id", &self.discord_id);
		d.insert("name", &self.name);
		d.insert("cards", &self.cards);

		d
	}

	pub fn is_valid(&self) -> bool {
		self.cards.len() == 60 as usize
	}
}