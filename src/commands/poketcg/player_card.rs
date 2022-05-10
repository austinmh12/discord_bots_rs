use serenity::builder::CreateEmbed;
use std::collections::HashMap;

use crate::commands::poketcg::card::{
	Card,
	get_multiple_cards_by_id
};

use super::{PaginateEmbed, CardInfo, Idable, HasSet, sets::Set};

#[derive(Clone)]
pub struct PlayerCard {
	pub card: Card,
	pub amount: i64
}

impl PaginateEmbed for PlayerCard {
	fn embed(&self) -> CreateEmbed {
		let mut e = self.card.embed();
		e
			.description(&self.description());

		e
	}
}

impl CardInfo for PlayerCard {
	fn card_id(&self) -> String {
		self.card.card_id.clone()
	}

	fn card_name(&self) -> String {
		self.card.name.clone()
	}

	fn description(&self) -> String {
		format!("**ID:** {}\n**Rarity:** {}\n**Price:** ${:.2}\n**Amount:** {}", &self.card.card_id, &self.card.rarity, &self.card.price, &self.amount)
	}
}

impl Idable for PlayerCard {
	fn id(&self) -> String {
		self.card.card_id.clone()
	}
}

impl HasSet for PlayerCard {
	fn set(&self) -> Set {
		self.card.set.clone()
	}
}

pub async fn player_cards(cards_hash: HashMap<String, i64>) -> Vec<PlayerCard> {
	let mut ret = vec![];
	let card_hash_clone = cards_hash.clone();
	let card_ids: Vec<String> = card_hash_clone.into_keys().collect();
	let cards = get_multiple_cards_by_id(card_ids.clone()).await;
	for card in cards {
		let amount = cards_hash.get(&card.id()).unwrap().to_owned();
		ret.push(PlayerCard {card, amount: amount.clone()});
	}

	ret
}