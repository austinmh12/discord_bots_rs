use dotenv;

async fn api_call(endpoint: &str, params: Option<&str>) -> Option<serde_json::Value> {
	dotenv::dotenv().ok();
	let poketcg_key = dotenv::var("POKETCGAPIKEY").unwrap();
	let client = reqwest::Client::new();

	let mut req = client
		.get(format!("https://api.pokemontcg.io/v2/{}", endpoint))
		.header("X-Api-Key", poketcg_key);
	req = match params {
		Some(x) => req.query(&[("q", x)]),
		None => req
	};
	let data: serde_json::Value = req
		.send().await.unwrap()
		.json().await.unwrap();
	

	if data.is_null() {
		None
	} else {
		Some(data)
	}
}

pub mod card;
pub mod sets;

// use std::{
// 	time::Duration,
// 	sync::{
// 		Arc,
// 	},
// 	collections::HashMap,
// };

use serenity::{framework::standard::{
	macros::{
		command,
	},
	Args,
	CommandResult
}, builder::CreateEmbed};
use serenity::model::{
	channel::{Message, ReactionType},
	//id::{ChannelId}
	//prelude::*,
};
use serenity::utils::Colour;
use serenity::prelude::*;
use serenity::collector::EventCollectorBuilder;
use std::time::Duration;

//use serenity::collector::MessageCollectorBuilder;
use serde_json;
use rand::seq::SliceRandom;

pub trait PaginateEmbed {
	fn embed(&self) -> CreateEmbed;
}

// paginated embeds to search through cards
async fn paginated_embeds(ctx: &Context, msg: &Message, embeds: Vec<CreateEmbed>) -> Result<(), String> {
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let mut idx: i16 = 0;
	// let mut cur_embed = &embeds[idx as usize].to_owned();
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			// let mut e = cur_card.embed();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			if embeds.len() > 1 {
				m.reactions([left_arrow.clone(), right_arrow.clone()]);
			}

			m			
		}).await.unwrap();
	
	loop {
		if embeds.len() <= 1 {
			break; // Exit before anything. Probably a way to do this before entering.
		}
		if let Some(reaction) = &message
			.await_reaction(&ctx)
			.timeout(Duration::from_secs(30))
			.author_id(msg.author.id)
			.removed(true)
			.await
		{
			let emoji = &reaction.as_inner_ref().emoji;
			match emoji.as_data().as_str() {
				"⬅️" => idx = (idx - 1).rem_euclid(embeds.len() as i16),
				"➡️" => idx = (idx + 1) % embeds.len() as i16,
				_ => continue
			};
		} else {
			message.delete_reactions(&ctx).await.expect("Couldn't remove arrows");
			break;
		}
		message.edit(&ctx, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
			if embeds.len() > 1 {
				cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
			}
			m.set_embed(cur_embed);

			m
		}).await.unwrap();
	}

	Ok(())
}

#[command("card")]
#[sub_commands(card_search, card_random)]
async fn card_main(ctx: &Context, msg: &Message) -> CommandResult {
	let card_help_str = "Here are the available **card** commands:
	**.card search**: Searches for a card with a matching name
	**.card random**: Shows a random card";
	msg.reply(&ctx.http, card_help_str).await?;

	Ok(())
}

#[command("search")]
async fn card_search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	println!("Calling search");
	let search_str = args.rest();
	let cards = card::get_cards_with_query(format!("name:{}", search_str).as_str()).await;
	let mut card_embeds = vec![];
	for card in cards {
		card_embeds.push(card.embed());
	}
	paginated_embeds(ctx, msg, card_embeds).await?;

	Ok(())
}

#[command("random")]
async fn card_random(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	println!("Calling search");
	let search_str = args.rest();
	// let left_arrow = ReactionType::try_from("⬅️").unwrap();
	// let right_arrow = ReactionType::try_from("➡️").unwrap();
	let cards = card::get_cards_with_query(format!("name:{}", search_str).as_str()).await;
	println!("Got cards: {}", cards.len());
	let cur_card = &cards.choose(&mut rand::thread_rng()).unwrap();
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(cur_card.name.clone())
					.description(format!("**ID:** {}\n**Price:** ${:.2}\n", cur_card.id.clone(), cur_card.price.clone()))
					.colour(Colour::from_rgb(255, 50, 20))
					.image(cur_card.image.clone())
			})
		}).await;

	Ok(())
}

#[command("set")]
#[sub_commands(search_set)]
async fn set_main(ctx: &Context, msg: &Message) -> CommandResult {
	let sets = sets::get_sets().await;
	let mut set_embeds = vec![];
	for set in sets {
		set_embeds.push(set.embed());
	}
	paginated_embeds(ctx, msg, set_embeds).await?;

	Ok(())
}

#[command("search")]
async fn search_set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	


	Ok(())
}

/* Command list
 * mycards
 * sell
 * 		card
 * 		under
 * 		dups
 * 		all
 * 		packs
 * search
 * sets
 * set
 * packs
 * openpack
 * store
 * stats
 * daily
 * quiz
 * savelist
 * 		add
 * 		remove
 * 		clear
 * trade
 * 		card
 * 		pack
*/

/* Tasks
 * Refresh daily packs
 * Cache store packs
 * Cache player cards
*/

/* Admin commands
 * cache
 * addcash
 * resetpacks
 * resetquiz
*/

/* Other things
 * Need to find a way to replicate the paginated embeds
 * 		I should look into how to work with generics so I can make a function like
 * 		async fn paginated_embeds(pages: Vec<T>)
 * Need to find a way to implement the store
 * Need to find a way to implement the cache
 * Need to learn how to add and wait for reactions
*/