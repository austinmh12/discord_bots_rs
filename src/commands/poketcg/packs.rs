use std::{
	collections::HashMap,
};
use rand::{
	prelude::*
};

use crate::sets::{
	Set,
	get_set
};
use crate::card::{
	Card,
	get_cards_with_query
};

lazy_static! {
	static ref RARITY_MAPPING: HashMap<&'static str, i64> = {
		let mut m = HashMap::new();
		m.insert("Rare", 75);
		m.insert("Rare ACE", 10);
		m.insert("Rare BREAK", 10);
		m.insert("Rare Holo", 25);
		m.insert("Rare Holo EX", 12);
		m.insert("Rare Holo GX", 12);
		m.insert("Rare Holo LV.X", 12);
		m.insert("Rare Holo Star", 8);
		m.insert("Rare Holo V", 15);
		m.insert("Rare Holo VMAX", 10);
		m.insert("Rare Prime", 10);
		m.insert("Rare Prism Star", 10);
		m.insert("Rare Rainbow", 5);
		m.insert("Rare Secret", 1);
		m.insert("Rare Shining", 20);
		m.insert("Rare Shiny", 5);
		m.insert("Rare Shiny GX", 2);
		m.insert("Rare Ultra", 35);
		m.insert("Amazing Rare", 15);
		m.insert("LEGEND", 3);
		
		m
	};
}

#[derive(Debug)]
pub struct Pack {
	pub set: Set,
	pub cards: Vec<Card>
}

impl Pack {
	pub async fn from_set_id(set_id: &str, amount: usize) -> Result<Self, String> {
		let set = get_set(set_id)
			.await
			.unwrap();
		let all_cards = get_cards_with_query(&format!("set.id:{}", set.id))
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
		let mut cards = vec![];
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

		Ok(Self {
			set,
			cards
		})
	}
}