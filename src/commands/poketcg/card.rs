use super::*;

pub struct Card {
	pub id: String,
	pub name: String,
	pub set_id: String, // This will eventually be a Set object
	pub number: String,
	pub price: f32,
	pub image: String
}

impl Card {
	pub fn new(id: String, name: String, set_id: String, number: String, price: f32, image: String) -> Self {
		Self {
			id,
			name,
			set_id,
			number,
			price,
			image
		}
	}
	pub fn from_json(obj: &serde_json::Value) -> Self {
		Self {
			id: String::from(obj["id"].as_str().unwrap()),
			name: String::from(obj["name"].as_str().unwrap()),
			set_id: String::from(obj["set"]["id"].as_str().unwrap()),
			number: String::from(obj["number"].as_str().unwrap()),
			price: 0.01,
			image: String::from(obj["images"]["large"].as_str().unwrap())
		}
	}
}

pub async fn get_cards() -> Vec<Card> {
	let mut ret = <Vec<Card>>::new();
	let data = api_call("cards", None).await.unwrap();
	let card_data = data["data"].as_array().unwrap();
	for cd in card_data {
		let card = Card::from_json(cd);
		ret.push(card);
	}

	ret
}

pub async fn get_cards_with_query(query: &str) -> Vec<Card> {
	let mut ret = <Vec<Card>>::new();
	let data = api_call("cards", Some(query)).await.unwrap();
	let card_data = data["data"].as_array().unwrap();
	for cd in card_data {
		let card = Card::from_json(cd);
		ret.push(card);
	}

	ret
}
