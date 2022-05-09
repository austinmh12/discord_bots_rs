use bson::Document;
use serde::{Serialize, Deserialize};
use super::{
	sets,
	card
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binder {
	pub set: String,
	pub cards: Vec<String>
}

impl Binder {
	pub fn empty() -> Self {
		Self {
			set: String::from(""),
			cards: vec![]
		}
	}

	pub fn from_set_id(set: String) -> Self {
		Self {
			set,
			cards: vec![]
		}
	}

	pub fn to_doc(&self) -> Document {
		let mut d = Document::new();
		d.insert("set", &self.set);
		d.insert("cards", &self.cards);

		d
	}

	pub async fn is_complete(&self) -> bool {
		let set = sets::get_set(&self.set).await.unwrap();
		let cards = card::get_cards_by_set(&set).await;

		cards.len() == self.cards.len()
	}

}