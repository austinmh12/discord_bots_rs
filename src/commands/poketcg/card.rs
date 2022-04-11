use super::{*, player::Player};
use crate::sets::Set;

#[derive(Clone, Debug)]
pub struct Card {
	pub id: String,
	pub name: String,
	pub set: Set, // This will eventually be a Set object
	pub number: String,
	pub price: f32,
	pub image: String,
	pub rarity: String
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
			id: String::from(obj["id"].as_str().unwrap()),
			name: String::from(obj["name"].as_str().unwrap()),
			set: Set::from_json(obj.get("set").unwrap()),
			number: String::from(obj["number"].as_str().unwrap()),
			price: price as f32,
			image: String::from(obj["images"]["large"].as_str().unwrap()),
			rarity
		}
	}
}

impl PaginateEmbed for Card {
	fn embed(&self) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		ret
			.title(&self.name)
			.description(format!("**ID:** {}\n**Rarity:** {}\n**Price:** ${:.2}\n", &self.id, &self.rarity, &self.price))
			.colour(Colour::from_rgb(255, 50, 20))
			.image(&self.image);

		ret
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

pub async fn get_multiple_cards_by_id(card_ids: Vec<String>) -> Vec<Card> {
	todo!();
	// For chunks of 250, join " OR " with "id:{}", card_id
	// return vec of cards
}

pub async fn get_card(id: &str) -> Card {
	let data = api_call(&format!("cards/{}", id), None)
		.await
		.unwrap();
	let card_data = &data["data"];
	let card = Card::from_json(&card_data);

	card
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
