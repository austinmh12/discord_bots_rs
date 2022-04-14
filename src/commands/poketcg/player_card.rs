use serenity::builder::CreateEmbed;
use std::collections::HashMap;

use crate::commands::poketcg::card::{
	Card,
	get_card,
	get_multiple_cards_by_id
};

use super::{PaginateEmbed, CardInfo};

pub struct PlayerCard {
	pub card: Card,
	pub amount: i64
}

impl PaginateEmbed for PlayerCard {
	fn embed(&self) -> CreateEmbed {
		let mut e = self.card.embed();
		e
			.description(format!("**ID:** {}\n**Rarity:** {}\n**Price:** ${:.2}\n**Amount:** {}\n", &self.card.id, &self.card.rarity, &self.card.price, &self.amount));

		e
	}
}

impl CardInfo for PlayerCard {
	fn card_id(&self) -> String {
		self.card.id.clone()
	}

	fn card_name(&self) -> String {
		self.card.name.clone()
	}
}

pub async fn player_card(card_id: &str, amount: i64) -> PlayerCard {
	let card = get_card(card_id).await;

	PlayerCard{card, amount}
}

pub async fn player_cards(cards_hash: HashMap<String, i64>) -> Vec<PlayerCard> {
	let mut ret = vec![];
	let card_hash_clone = cards_hash.clone();
	let card_ids: Vec<String> = card_hash_clone.into_keys().collect();
	let cards = get_multiple_cards_by_id(card_ids.clone()).await;
	for card in cards {
		let amount = cards_hash.get(&card.id).unwrap().to_owned();
		ret.push(PlayerCard {card, amount: amount.clone()});
	}

	ret
}