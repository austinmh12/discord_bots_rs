use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Upgrade {
	pub daily_time_reset: i64,
	pub daily_reward_mult: i64,
	pub daily_pack_amount: i64,
	pub store_discount: i64,
	pub tokenshop_discount: i64,
	pub slot_reward_mult: i64,
	pub daily_slot_amount: i64
}

impl Upgrade {
	pub fn new() -> Self {
		Self {
			daily_time_reset: 0,
			daily_reward_mult: 0,
			daily_pack_amount: 0,
			store_discount: 0,
			tokenshop_discount: 0,
			slot_reward_mult: 0,
			daily_slot_amount: 0
		}
	}

	pub fn upgrade_cost(&self, upgrade: &str) -> f64 {
		match upgrade {
			"daily_time_reset" => 200.0,
			"daily_reward_mult" => 100.0 + (100.0 * self.daily_reward_mult as f64),
			"daily_pack_amount" => (1000 + (0..self.daily_pack_amount).iter().map(|i| i * 1000).sum::<i64>()) as f64,
			"store_discount" => 250.0 + (250.0 * self.store_discount as f64),
			"tokenshop_discount" => 250.0 + (250.0 * self.tokenshop_discount as f64),
			"slot_reward_mult" => 500.0 + (500.0 * self.slot_reward_mult as f64),
			"daily_slot_amount" => 750.0 + (750.0 * self.daily_slot_amount as f64),
			_ => 0.0
		}
	}
}