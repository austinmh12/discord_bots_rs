use rand::{
	prelude::*
};
use image::{
	io::Reader,
	Rgba
};
use std::io::Cursor;
use serenity::{
	framework::{
		standard::{
			macros::{
				command
			},
			CommandResult
		},
	},
	model::{
		channel::{
			Message,
		},
	},
	prelude::*
};
use chrono::{
	Utc,
	Duration,
	DateTime,
	Local,
};
use std::time::Duration as StdDuration;
use crate::{
	player,
};
use mongodb::bson::{Document, doc};
use convert_case::{Case, Casing};

const NAME_TO_KEEP_DASH: &'static [&str] = &[
	"ho-oh",
	"porygon-z",
	"type-null",
	"jangmo-o",
	"hakamo-o",
	"kommo-o"
];
const NAMES_TO_REPLACE_DASH: &'static [&str] = &[
	"mr-mime",
	"mr-rime",
	"mime-jr",
	"tapu-koko",
	"tapu-lele",
	"tapu-bulu",
	"tapu-fini"
];
pub struct Quiz {
	pub national_id: i64,
	pub name: String,
	pub generation: i64
}

impl Quiz {
	pub async fn random_quiz() -> Self {
		let national_id = rand::thread_rng().gen_range(1..=898) as i64;
		let resp: serde_json::Value = reqwest::Client::new()
			.get(format!("https://pokeapi.co/api/v2/pokemon/{}", &national_id))
			.send().await.unwrap()
			.json().await.unwrap();
		let name = String::from(resp["name"].as_str().unwrap());
		let generation = match national_id {
			1..=151 => 1,
			152..=251 => 2,
			252..=386 => 3,
			387..=493 => 4,
			494..=649 => 5,
			650..=721 => 6,
			722..=809 => 7,
			_ => 8
		};

		Self {
			national_id,
			name,
			generation
		}
	}

	pub async fn generate_silhouette(&self, light_mode: bool) {
		let resp = reqwest::Client::new()
			.get(format!("https://img.pokemondb.net/sprites/home/normal/{}.png", self.name))
			.send().await.unwrap()
			.bytes().await.unwrap();
		let reader = Reader::new(Cursor::new(resp))
			.with_guessed_format()
			.expect("Can't get image");
		let mut image = reader.decode().unwrap().into_rgba8();
		image.save("quizresult.PNG").unwrap();
		let (width, height) = image.dimensions();
		for i in 0..width {
			for j in 0..height {
				let px = image.get_pixel(i, j);
				if px.0[3] != 0 {
					if light_mode {
						image.put_pixel(i, j, Rgba([255, 255, 255, 255]));
					} else {
						image.put_pixel(i, j, Rgba([0, 0, 0, 255]));
					}
				}
			}
		}
		image.save("quizsilhouette.PNG").unwrap();
	}

	pub fn guess_name(&self) -> String {
		if NAME_TO_KEEP_DASH.contains(&self.name.as_str()) {
			return self.name.clone();
		}
		let mut name = self.name.clone();
		if NAMES_TO_REPLACE_DASH.contains(&name.as_str()) {
			name = name.replace("-", " ").to_string();
		}
		if name.contains("-") {
			name = name.split("-").collect::<Vec<&str>>()[0].to_string();
		}

		name
	}
}

#[command("quiz")]
#[aliases("q")]
async fn quiz_command(ctx: &Context, msg: &Message) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	if player.quiz_reset < Utc::now() {
		player.quiz_questions = 5 + player.upgrades.quiz_question_amount;
		let minutes_til_reset = 120 - 10 * player.upgrades.quiz_time_reset;
		player.quiz_reset = Utc::now() + Duration::minutes(minutes_til_reset);
	}
	if player.quiz_questions <= 0 {
		let local_timer: DateTime<Local> = DateTime::from(player.quiz_reset);
		msg.reply(&ctx.http, format!("Your quiz attempts reset **{}**", local_timer.format("%h %d %H:%M"))).await?;
		return Ok(());
	}
	let quiz = Quiz::random_quiz().await;
	quiz.generate_silhouette(player.light_mode).await;
	let mut quiz_msg = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m
				.content("Who's that Pokemon?!")
				.add_file("./quizsilhouette.PNG")
		})
		.await?;
	let attachment_id = quiz_msg.attachments[0].id;
	if let Some(quiz_reply) = &msg.author.await_reply(&ctx).timeout(StdDuration::from_secs(15)).await {
		let guess = &quiz_reply.content.to_lowercase();
		let gen_guess = match guess.parse::<i64>() {
			Ok(x) => x,
			Err(_) => 0
		};
		if guess == &quiz.guess_name() {
			let reward = 0.1 * player.current_multiplier as f64;
			player.quiz_correct += 1;
			if player.current_multiplier < player.perm_multiplier + (player.upgrades.quiz_mult_limit * 10) {
				player.current_multiplier += 1;
			}
			player.cash += reward;
			player.total_cash += reward;
			quiz_msg.edit(&ctx.http, |m| {
				m
					.content(format!("Correct! It's **{}**\nYou earned **${:.2}** and your multiplier is now **{}**", quiz.name.to_case(Case::Title), reward, player.current_multiplier))
					.remove_existing_attachment(attachment_id)
					.attachment("./quizresult.PNG")
			})
			.await?;

		} else if gen_guess == quiz.generation {
			let reward = 0.1 * player.current_multiplier as f64;
			player.quiz_correct += 1;
			player.cash += reward;
			player.total_cash += reward;
			quiz_msg.edit(&ctx.http, |m| {
				m
					.content(format!("Sure, it's from **Gen {}**. It's **{}**\nYou earned **${:.2}**", quiz.generation, quiz.name.to_case(Case::Title), reward))
					.remove_existing_attachment(attachment_id)
					.attachment("./quizresult.PNG")
			})
			.await?;
		} else if guess == "pikachu" && &quiz.guess_name() == "clefairy" {
			let reward = 1.0 * player.current_multiplier as f64;
			player.quiz_correct += 1;
			player.cash += reward;
			player.total_cash += reward;
			quiz_msg.edit(&ctx.http, |m| {
				m
					.content("It's **Clefairy**! ***FUCK***")
					.remove_existing_attachment(attachment_id)
					.attachment("./quizresult.PNG")
			})
			.await?;
		} else {
			quiz_msg.edit(&ctx.http, |m| {
				m
					.content(format!("Wrong! It's **{}** from **Gen {}**", quiz.name.to_case(Case::Title), quiz.generation))
					.remove_existing_attachment(attachment_id)
					.attachment("./quizresult.PNG")
			})
			.await?;
		}
	} else {
		quiz_msg.edit(&ctx.http, |m| {
			m
				.content(format!("You ran out of time, it's **{}** from **Gen {}**", quiz.name.to_case(Case::Title), quiz.generation))
				.remove_existing_attachment(attachment_id)
				.attachment("./quizresult.PNG")
		})
		.await?;
	}
	let mut player_update = Document::new();
	player.quiz_questions -= 1;
	player_update.insert("quiz_correct", player.quiz_correct);
	player_update.insert("current_multiplier", player.current_multiplier);
	player_update.insert("cash", player.cash);
	player_update.insert("total_cash", player.total_cash);
	player_update.insert("quiz_questions", player.quiz_questions);
	player_update.insert("quiz_reset", player.quiz_reset);
	player::update_player(&player, doc! { "$set": player_update}).await;

	Ok(())
}