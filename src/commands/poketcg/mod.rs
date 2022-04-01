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

// #[command("card")]
// #[sub_commands(card_search, card_random)]
// async fn card_main(ctx: &Context, msg: &Message) -> CommandResult {
// 	let card_help_str = "Here are the available **card** commands:
// 	**.card search**: Searches for a card with a matching name
// 	**.card random**: Shows a random card";
// 	msg.reply(&ctx.http, card_help_str).await?;

// 	Ok(())
// }

// #[command("search")]
// async fn card_search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
// 	println!("Calling search");
// 	let search_str = args.rest();
// 	let cards = card::get_cards_with_query(format!("name:{}", search_str).as_str()).await;
// 	let mut card_embeds = vec![];
// 	for card in cards {
// 		card_embeds.push(card.embed());
// 	}
// 	paginated_embeds(ctx, msg, card_embeds).await?;

// 	Ok(())
// }

// #[command("set")]
// #[sub_commands(search_set)]
// async fn set_main(ctx: &Context, msg: &Message) -> CommandResult {
// 	let sets = sets::get_sets().await;
// 	let mut set_embeds = vec![];
// 	for set in sets {
// 		set_embeds.push(set.embed());
// 	}
// 	paginated_embeds(ctx, msg, set_embeds).await?;

// 	Ok(())
// }

#[command("my")]
#[sub_commands(my_cards, my_packs, my_stats)]
async fn my_main(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("cards")]
async fn my_cards(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("packs")]
async fn my_packs(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("stats")]
async fn my_stats(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("sell")]
#[sub_commands(sell_card, sell_under, sell_dups, sell_all, sell_packs)]
async fn sell_main(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("card")]
async fn sell_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("under")]
async fn sell_under(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("dups")]
async fn sell_dups(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("all")]
async fn sell_all(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("packs")]
async fn sell_packs(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("search")]
#[sub_commands(search_card, search_set)]
async fn search_main(ctx: &Context, msg: &Message) -> CommandResult {
	let search_help_str = "Here are the available **search** commands:
	**.search card:** Searches for a card with a matching name
	**.search set:** Searches for a set with a matching name";
	msg.reply(&ctx.http, search_help_str).await?;

	Ok(())
}

#[command("card")]
async fn search_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let search_str = args.rest();
	let cards = card::get_cards_with_query(&format!("name:{}", search_str))
		.await;
	if cards.len() == 0 {
		msg.reply(&ctx.http, "No cards found with that name.").await?;
	} else {
		let embeds = cards.iter().map(|c| c.embed()).collect::<Vec<_>>();
		paginated_embeds(ctx, msg, embeds).await?;
	}

	Ok(())
}
// NtS: Maybe make .search card name <cardName> and .search card id <cardId> ?
// NtS: (N)ote (t)o (S)elf

#[command("set")]
async fn search_set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let search_str = args.rest();
	let sets = sets::get_sets_with_query(&format!("name:{}", search_str))
		.await;
	if sets.len() == 0 {
		msg.reply(&ctx.http, "No sets found with that name.").await?;
	} else {
		let embeds = sets.iter().map(|c| c.embed()).collect::<Vec<_>>();
		paginated_embeds(ctx, msg, embeds).await?;
	}

	Ok(())
}

#[command("sets")]
async fn sets_command(ctx: &Context, msg: &Message) -> CommandResult {
	

	Ok(())
}

#[command("set")]
async fn set_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	

	Ok(())
}

#[command("openpack")]
#[aliases("op")]
async fn open_pack_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("store")]
async fn store_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	

	Ok(())
}

#[command("daily")]
#[aliases("d")]
async fn daily_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("quiz")]
async fn quiz_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	

	Ok(())
}

#[command("savelist")]
#[aliases("sl")]
#[sub_commands(savelist_add, savelist_list, savelist_clear, savelist_remove)]
async fn savelist_main(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("list")]
async fn savelist_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("add")]
async fn savelist_add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("remove")]
async fn savelist_remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("clear")]
async fn savelist_clear(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("trade")]
#[sub_commands(trade_card, trade_pack)]
async fn trade_main(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("card")]
async fn trade_card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("pack")]
async fn trade_pack(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	

	Ok(())
}

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
*/