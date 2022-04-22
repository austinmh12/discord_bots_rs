use crate::{
	player::{
		Player
	},
};

#[derive(Debug)]
pub struct Trade {
	pub cash: f64,
	pub cards: Vec<(String, i64)>,
	pub packs: Vec<(String, i64)>
}

impl Trade {
	pub fn from_trade_str(trade_str: &str) -> Self {
		let offers = trade_str.split("/").collect::<Vec<&str>>();
		let mut cash = 0.00;
		let mut cards = vec![];
		let mut packs = vec![];
		for offer in offers {
			if offer.contains("$") {
				let amt = offer
					.replace("$", "")
					.parse::<f64>()
					.unwrap_or(0.00);
				cash += amt;
			} else if offer.contains("-") {
				let card_amount = offer
					.split(":")
					.collect::<Vec<&str>>();
				let card = String::from(card_amount[0]);
				if card_amount.len() == 1 {
					cards.push((card, 1));
				} else {
					let amt = card_amount[1].parse::<i64>().unwrap_or(1);
					cards.push((card, amt));
				}
			} else {
				let pack_amount = offer
					.split(":")
					.collect::<Vec<&str>>();
				let pack = String::from(pack_amount[0]);
				if pack_amount.len() == 1 {
					packs.push((pack, 1));
				} else {
					let amt = pack_amount[1].parse::<i64>().unwrap_or(1);
					packs.push((pack, amt));
				}
			}
		}

		Self {
			cash,
			cards,
			packs
		}
	}

	pub fn player_has_all(&self, player: &Player) -> bool {
		if player.cash < self.cash {
			return false;
		}
		for (card_id, amt) in &self.cards {
			if player.cards.contains_key(card_id) {
				if player.cards.get(card_id).unwrap() < &amt {
					return false;
				}
			} else {
				return false;
			}
		}
		for (pack_id, amt) in &self.packs {
			if player.packs.contains_key(pack_id) {
				if player.packs.get(pack_id).unwrap() < &amt {
					return false;
				}
			} else {
				return false;
			}
		}

		true
	}
}