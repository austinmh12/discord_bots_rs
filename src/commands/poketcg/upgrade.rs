use bson::Document;
use serde::{Serialize, Deserialize};
use serenity::{builder::CreateEmbed, utils::Colour};

use super::{player::Player};

fn def_0() -> i64 {
	0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
	pub daily_time_reset: i64,
	pub daily_reward_mult: i64,
	pub daily_pack_amount: i64,
	pub store_discount: i64,
	pub tokenshop_discount: i64,
	pub slot_reward_mult: i64,
	pub daily_slot_amount: i64,
	#[serde(default = "def_0")]
	pub quiz_time_reset: i64,
	#[serde(default = "def_0")]
	pub quiz_question_amount: i64,
	#[serde(default = "def_0")]
	pub quiz_mult_limit: i64
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
			daily_slot_amount: 0,
			quiz_time_reset: 0,
			quiz_question_amount: 0,
			quiz_mult_limit: 0
		}
	}

	pub fn upgrade_cost(&self, upgrade: &str) -> f64 {
		match upgrade {
			"daily_time_reset" => 200.0,
			"daily_reward_mult" => 100.0 + (100.0 * self.daily_reward_mult as f64),
			"daily_pack_amount" => (1000 + (0..=self.daily_pack_amount).map(|i| i * 1000).sum::<i64>()) as f64,
			"store_discount" => 250.0 + (250.0 * self.store_discount as f64),
			"tokenshop_discount" => 250.0 + (250.0 * self.tokenshop_discount as f64),
			"slot_reward_mult" => 500.0 + (500.0 * self.slot_reward_mult as f64),
			"daily_slot_amount" => 750.0 + (750.0 * self.daily_slot_amount as f64),
			"quiz_time_reset" =>  300.0,
			"quiz_question_amount" =>  500.0 + (100.0 * self.quiz_question_amount as f64),
			"quiz_mult_limit" => 400.0 + (250.0 * self.quiz_mult_limit as f64),
			_ => 0.0
		}
	}

	pub fn is_max_upgrade(&self, upgrade: &str) -> bool {
		match upgrade {
			"daily_time_reset" => self.daily_time_reset >= 12,
			"daily_reward_mult" => self.daily_reward_mult >= 20,
			"daily_pack_amount" => self.daily_pack_amount >= 5,
			"store_discount" => self.store_discount >= 5,
			"tokenshop_discount" => self.tokenshop_discount >= 5,
			"slot_reward_mult" => self.slot_reward_mult >= 10,
			"daily_slot_amount" => self.daily_slot_amount >= 10,
			"quiz_time_reset" => self.quiz_time_reset >= 9,
			"quiz_question_amount" => self.quiz_question_amount >= 5,
			"quiz_mult_limit" => self.quiz_mult_limit >= 5,
			_ => false
		}
	}

	pub async fn embed_with_player(&self, player: Player) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		let mut desc = String::from("Welcome to the Upgrade Store! Here you can spend cash for various upgrades\n");
		desc.push_str(&format!("You have **${:.2}**\n", player.cash));
		desc.push_str("Here are the upgrades available. To purchase an upgrade, use **.(up)grades (b)uy <slot no. | name.> [amount]**\n\n");
		if !self.is_max_upgrade("daily_time_reset") {
			desc.push_str(&format!("**1 dailytime:** Decreases the time between daily resets - ${:.2} _(lvl {})_\n", self.upgrade_cost("daily_time_reset"), self.daily_time_reset));
		}
		if !self.is_max_upgrade("daily_reward_mult") {
			desc.push_str(&format!("**2 dailyreward:** Increases your daily rewards - ${:.2} _(lvl {})_\n", self.upgrade_cost("daily_reward_mult"), self.daily_reward_mult));
		}
		if !self.is_max_upgrade("daily_pack_amount") {
			desc.push_str(&format!("**3 dailypacks:** Increases your daily packs - ${:.2} _(lvl {})_\n", self.upgrade_cost("daily_pack_amount"), self.daily_pack_amount));
		}
		if !self.is_max_upgrade("store_discount") {
			desc.push_str(&format!("**4 storediscount:** Gives a discount at the store - ${:.2} _(lvl {})_\n", self.upgrade_cost("store_discount"), self.store_discount));
		}
		if !self.is_max_upgrade("tokenshop_discount") {
			desc.push_str(&format!("**5 tokenshopdiscount:** Gives a discount at the token shop - ${:.2} _(lvl {})_\n", self.upgrade_cost("tokenshop_discount"), self.tokenshop_discount));
		}
		if !self.is_max_upgrade("slot_reward_mult") {
			desc.push_str(&format!("**6 slotreward:** Increases your slot machine rewards - ${:.2} _(lvl {})_\n", self.upgrade_cost("slot_reward_mult"), self.slot_reward_mult));
		}
		if !self.is_max_upgrade("daily_slot_amount") {
			desc.push_str(&format!("**7 dailyslots:** Increases your daily slots - ${:.2} _(lvl {})_\n", self.upgrade_cost("daily_slot_amount"), self.daily_slot_amount));
		}
		if !self.is_max_upgrade("quiz_time_reset") {
			desc.push_str(&format!("**8 quizreset:** Decreases the time between your quiz attempts - ${:.2} _(lvl {})_\n", self.upgrade_cost("quiz_time_reset"), self.quiz_time_reset));
		}
		if !self.is_max_upgrade("quiz_question_amount") {
			desc.push_str(&format!("**9 quizattempts:** Increases your quiz questions per reset period - ${:.2} _(lvl {})_\n", self.upgrade_cost("quiz_question_amount"), self.quiz_question_amount));
		}
		if !self.is_max_upgrade("quiz_mult_limit") {
			desc.push_str(&format!("**10 quizmultiplier:** Increases your max quiz multiplier - ${:.2} _(lvl {})_\n", self.upgrade_cost("quiz_mult_limit"), self.quiz_mult_limit));
		}
		ret
			.title("Upgrade Shop")
			.description(&desc)
			.colour(Colour::from_rgb(255, 50, 20));

		ret
	}

	pub fn to_doc(&self) -> Document {
		let mut d = Document::new();
		d.insert("daily_time_reset", self.daily_time_reset);
		d.insert("daily_reward_mult", self.daily_reward_mult);
		d.insert("daily_pack_amount", self.daily_pack_amount);
		d.insert("store_discount", self.store_discount);
		d.insert("tokenshop_discount", self.tokenshop_discount);
		d.insert("slot_reward_mult", self.slot_reward_mult);
		d.insert("daily_slot_amount", self.daily_slot_amount);
		d.insert("quiz_time_reset", self.quiz_time_reset);
		d.insert("quiz_question_amount", self.quiz_question_amount);
		d.insert("quiz_mult_limit", self.quiz_mult_limit);

		d
	}

	pub fn desc(&self) -> String {
		let mut ret = String::from("");
		match self.is_max_upgrade("daily_time_reset") {
			true => ret.push_str(&format!("**dailytime:** {} ***MAX***\n", self.daily_time_reset)),
			false => ret.push_str(&format!("**dailytime:** {}\n", self.daily_time_reset))
		}
		match self.is_max_upgrade("daily_reward_mult") {
			true => ret.push_str(&format!("**dailyreward:** {} ***MAX***\n", self.daily_reward_mult)),
			false => ret.push_str(&format!("**dailyreward:** {}\n", self.daily_reward_mult))
		}
		match self.is_max_upgrade("daily_pack_amount") {
			true => ret.push_str(&format!("**dailypacks:** {} ***MAX***\n", self.daily_pack_amount)),
			false => ret.push_str(&format!("**dailypacks:** {}\n", self.daily_pack_amount))
		}
		match self.is_max_upgrade("store_discount") {
			true => ret.push_str(&format!("**storediscount:** {} ***MAX***\n", self.store_discount)),
			false => ret.push_str(&format!("**storediscount:** {}\n", self.store_discount))
		}
		match self.is_max_upgrade("tokenshop_discount") {
			true => ret.push_str(&format!("**tokenshopdiscount:** {} ***MAX***\n", self.tokenshop_discount)),
			false => ret.push_str(&format!("**tokenshopdiscount:** {}\n", self.tokenshop_discount))
		}
		match self.is_max_upgrade("slot_reward_mult") {
			true => ret.push_str(&format!("**slotreward:** {} ***MAX***\n", self.slot_reward_mult)),
			false => ret.push_str(&format!("**slotreward:** {}\n", self.slot_reward_mult))
		}
		match self.is_max_upgrade("daily_slot_amount") {
			true => ret.push_str(&format!("**dailyslots:** {} ***MAX***\n", self.daily_slot_amount)),
			false => ret.push_str(&format!("**dailyslots:** {}\n", self.daily_slot_amount))
		}
		match self.is_max_upgrade("quiz_time_reset") {
			true => ret.push_str(&format!("**quizreset:** {} ***MAX***\n", self.quiz_time_reset)),
			false => ret.push_str(&format!("**quizreset:** {}\n", self.quiz_time_reset))
		}
		match self.is_max_upgrade("quiz_question_amount") {
			true => ret.push_str(&format!("**quizattempts:** {} ***MAX***\n", self.quiz_question_amount)),
			false => ret.push_str(&format!("**quizattempts:** {}\n", self.quiz_question_amount))
		}
		match self.is_max_upgrade("quiz_mult_limit") {
			true => ret.push_str(&format!("**quizmultiplier:** {} ***MAX***\n", self.quiz_mult_limit)),
			false => ret.push_str(&format!("**quizmultiplier:** {}\n", self.quiz_mult_limit))
		}

		ret
	}
}