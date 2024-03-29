use rand::{
	prelude::*
};
use serenity::prelude::Context;

use crate::sets::{
	Set,
	get_set
};
use crate::card::{
	Card,
	get_cards_by_set
};
use crate::commands::poketcg::RARITY_MAPPING;

#[derive(Debug)]
pub struct Pack {
	pub set: Set,
	pub cards: Vec<Card>
}

impl Pack {
	pub async fn from_set_id(ctx: &Context, set_id: &str, amount: usize) -> Result<Self, String> {
		let set = get_set(set_id)
			.await
			.unwrap();
		let all_cards = get_cards_by_set(ctx, &set)
			.await;
		let rares = all_cards
			.iter()
			.filter(|c| c.rarity != "Common" || c.rarity != "Uncommon" || c.rarity != "Promo")
			.map(|c| c.to_owned())
			.collect::<Vec<Card>>();
		let uncommons = all_cards
			.iter()
			.filter(|c| c.rarity == "Uncommon")
			.map(|c| c.to_owned())
			.collect::<Vec<Card>>();
		let commons = all_cards
			.iter()
			.filter(|c| c.rarity == "Common")
			.map(|c| c.to_owned())
			.collect::<Vec<Card>>();
		let promos = all_cards
			.iter()
			.filter(|c| c.rarity == "Promo" || c.rarity == "Classic Collection" || c.rarity == "Unknown")
			.map(|c| c.to_owned())
			.collect::<Vec<Card>>();
		let mut cards = vec![];
		if commons.len() > 0 && uncommons.len() > 0 && rares.len() > 0 {
			if commons.len() > 0 {
				for _ in 0..6*amount {
					let c = commons.choose(&mut thread_rng()).unwrap().clone();
					cards.push(c);
				}
			}
			if uncommons.len() > 0 {
				for _ in 0..3*amount {
					let c = uncommons.choose(&mut thread_rng()).unwrap().clone();
					cards.push(c);
				}
			}
			let mut rares_with_weights = vec![];
			for rare in rares {
				let weight = RARITY_MAPPING.get(rare.rarity.as_str()).unwrap_or(&0).to_owned();
				rares_with_weights.push((rare, weight));
			}
			for _ in 0..amount {
				let c = rares_with_weights
					.choose_weighted(&mut thread_rng(), |rw| rw.1 as i32)
					.unwrap()
					.clone()
					.0;
				cards.push(c);
			}
		} else {
			for _ in 0..amount {
				let c = promos.choose(&mut thread_rng()).unwrap().clone();
				cards.push(c);
			}
		}

		Ok(Self {
			set,
			cards
		})
	}
}