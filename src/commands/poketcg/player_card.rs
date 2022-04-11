use serenity::builder::CreateEmbed;

use crate::commands::poketcg::card::{
	Card,
	get_card
};

use super::PaginateEmbed;

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

pub async fn player_card(card_id: &str, amount: i64) -> PlayerCard {
	let card = get_card(card_id).await;

	PlayerCard{card, amount}
}