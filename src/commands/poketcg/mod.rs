use chrono::{
	Utc,
	Duration
};
use dotenv;
use mongodb::{
	bson::{
		doc,
		Document
	},
};
use std::time::Duration as StdDuration;
pub mod card;
pub mod sets;
use sets::get_set;
pub mod packs;
pub mod player;
pub mod store;

use serenity::{
	framework::{
		standard::{
			macros::{
				command
			},
			Args,
			CommandResult
		},
	},
	builder::{
		CreateEmbed
	},
	model::{
		channel::{
			Message,
			ReactionType
		},
	},
	utils::{
		Colour
	},
	prelude::*
};

use serde_json;
use rand::{
	Rng
};
use crate::OWNER_CHECK;

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

pub trait PaginateEmbed {
	fn embed(&self) -> CreateEmbed;
}

// paginated embeds to search through cards
async fn paginated_embeds<T:PaginateEmbed>(ctx: &Context, msg: &Message, embeds: Vec<T>) -> Result<(), String> {
	let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
	let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
	let embeds = embeds.iter().map(|e| e.embed()).collect::<Vec<_>>();
	let mut idx: i16 = 0;
	let mut message = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			let mut cur_embed = embeds[idx as usize].clone();
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
			.timeout(StdDuration::from_secs(30))
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
	let player_ = player::get_player(msg.author.id.0).await;
	msg.reply(&ctx.http, format!("Hello {}, you have **${}**", player_.discord_id, player_.cash))
		.await?;

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
		paginated_embeds(ctx, msg, cards).await?;
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
		paginated_embeds(ctx, msg, sets).await?;
	}

	Ok(())
}

#[command("sets")]
async fn sets_command(ctx: &Context, msg: &Message) -> CommandResult {
	let sets = sets::get_sets().await;
	paginated_embeds(ctx, msg, sets).await?;

	Ok(())
}

#[command("set")]
async fn set_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let set_id = args.rest();
	let set = sets::get_set(set_id).await;
	match set {
		Some(x) => paginated_embeds(ctx, msg, vec![x]).await?,
		None => {
			msg.reply(&ctx.http, "No set found with that id.").await?;
		}
	}

	Ok(())
}

#[command("openpack")]
#[aliases("op")]
async fn open_pack_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// TODO: Set up database

	Ok(())
}

#[command("store")]
#[sub_commands(store_buy)]
async fn store_main(ctx: &Context, msg: &Message) -> CommandResult {
	let store_ = store::get_store().await;
	let player_ = player::get_player(msg.author.id.0).await;
	let embed = store_.embed_with_player(player_).await;
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.set_embed(embed);

			m
		})
		.await;

	Ok(())
}

#[command("buy")]
async fn store_buy(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let selection = match args.single::<i32>() {
		Ok(x) => x,
		Err(_) => 0
	};
	if !(1..=10).contains(&selection) {
		msg.channel_id.send_message(&ctx.http, |m| m.content("A selection was not made.")).await?;
		return Ok(());
	}
	let amount = match args.single::<i32>() {
		Ok(x) => x,
		Err(_) => 1
	};
	let store_ = store::get_store().await;
	let set = get_set(store_.sets.get((selection - 1) as usize).unwrap()).await.unwrap();
	let mut player = player::get_player(msg.author.id.0).await;
	let (price_mult, pack_count) = if selection <= 4 {
		(1.0, 1)
	} else if 5 <= selection && selection <= 7 {
		(2.5, 4)
	} else if 8 <= selection && selection <= 9 {
		(10.0, 12)
	} else {
		(30.0, 36)
	};
	if player.cash < set.pack_price() * price_mult {
		msg.channel_id.send_message(&ctx.http, |m| m.content(&format!("You don't have enough... You need **${:.2}** more", set.pack_price() * price_mult - player.cash))).await?;
		return Ok(());
	}
	let mut bought = 0;
	while player.cash >= set.pack_price() * price_mult && bought < amount {
		player.cash -= set.pack_price() * price_mult;
		bought += 1;
	}
	*player.packs.entry(set.id).or_insert(0) += (bought * pack_count) as i64;
	player.packs_bought += (bought * pack_count) as i64;
	msg.channel_id.send_message(&ctx.http, |m| m.content(&format!("You bought {} **{}** packs!", bought * pack_count, set.name))).await?;
	let mut player_packs = Document::new();
	for (set_id, amt) in player.packs.iter() {
		player_packs.insert(set_id, amt.clone());
	}
	player::update_player(
		&player,
		doc! {
			"$set": {
				"cash": &player.cash,
				"packs_bought": &player.packs_bought,
				"packs": player_packs
			}
		}
	).await;

	Ok(())
}

#[command("daily")]
#[aliases("d")]
async fn daily_command(ctx: &Context, msg: &Message) -> CommandResult {
	let mut player = player::get_player(msg.author.id.0).await;
	let now = Utc::now();
	if player.daily_reset >= now {
		msg
			.channel_id
			.send_message(
				&ctx.http, |m| m
					.content(
						format!("Your daily resets at {}", player.daily_reset.format("%Y/%m/%d %H:%M:%S"))
				)
			)
			.await
			.unwrap();
		return Ok(());
	}
	let mut update = Document::new();
	let r: i64 = rand::thread_rng().gen_range(0..100);
	if r <= 1 {
		player.daily_packs = 50;
		update.insert("daily_packs", player.daily_packs);
		msg.reply(&ctx.http, "***WOAH!*** Your daily packs were reset!").await?;
	} else {
		let cash: i64 = rand::thread_rng().gen_range(5..=20);
		player.cash += cash as f64;
		player.total_cash += cash as f64;
		update.insert("cash", player.cash);
		update.insert("total_cash", player.total_cash);
		msg.reply(&ctx.http, format!("You got **${:.2}**", cash as f64)).await?;
	}
	player.daily_reset = Utc::now() + Duration::days(1);
	update.insert("daily_reset", player.daily_reset);
	player::update_player(&player, doc!{ "$set": update }).await;

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

#[command("admin")]
#[sub_commands(admin_show_pack, admin_add_cash)]
#[checks(Owner)]
// TODO: Add the owner or admin check
async fn admin_main() -> CommandResult {
	Ok(())
}

#[command("pack")]
#[checks(Owner)]
async fn admin_show_pack(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let set_id = args.find::<String>().unwrap();
	let amount = match args.find::<i32>() {
		Ok(x) => x as usize,
		Err(_) => 1usize
	};
	let pack = packs::Pack::from_set_id(set_id.as_str(), amount).await.unwrap();
	paginated_embeds(ctx, msg, pack.cards).await.unwrap();

	Ok(())
}

#[command("cash")]
#[checks(Owner)]
async fn admin_add_cash(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let mut player_ = player::get_player(msg.author.id.0).await;
	let amount = args.find::<f64>().expect("No amount to add");
	let og_cash = player_.cash;
	player_.cash += amount;
	player_.total_cash += amount;
	msg.reply(&ctx.http, format!("{} had **${:.2}**, now they have **${:.2}**", &player_.discord_id, og_cash, &player_.cash))
		.await?;
	player::update_player(
		&player_,
		doc! {
			"$set": { 
				"cash": &player_.cash,
				"total_cash": &player_.total_cash
			}
		}
	)
		.await;

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