use bson::Document;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binder {
	pub set: String,
	pub cards: Vec<String>
}

impl Binder {
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

}