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
		},
	},
	utils::{
		Colour
	},
	prelude::*
};
use super::{
	*,
};
use std::collections::HashMap;
use async_trait::async_trait;

use crate::commands::poketcg::card::{
	Card,
	get_multiple_cards_by_id
};

use super::{PaginateEmbed, CardInfo, Idable, HasSet, sets::Set, Scrollable};

#[derive(Clone)]
pub struct PlayerCard {
	pub card: Card,
	pub amount: i64
}

impl PaginateEmbed for PlayerCard {
	fn embed(&self) -> CreateEmbed {
		let mut e = self.card.embed();
		e
			.description(&self.description());

		e
	}
}

impl CardInfo for PlayerCard {
	fn card_id(&self) -> String {
		self.card.card_id.clone()
	}

	fn card_name(&self) -> String {
		self.card.name.clone()
	}

	fn description(&self) -> String {
		format!("**ID:** {}\n**Rarity:** {}\n**Price:** ${:.2}\n**Amount:** {}", &self.card.card_id, &self.card.rarity, &self.card.price, &self.amount)
	}

	fn price(&self) -> f64 {
		self.card.price.clone()
	}
}

impl Idable for PlayerCard {
	fn id(&self) -> String {
		self.card.card_id.clone()
	}
}

impl HasSet for PlayerCard {
	fn set(&self) -> Set {
		self.card.set.clone()
	}
}

#[async_trait]
impl Scrollable for Vec<PlayerCard> {
	async fn scroll_through(&self, ctx: &Context, msg: &Message) -> Result<(), String> {
		let left_arrow = ReactionType::try_from("⬅️").expect("No left arrow");
		let right_arrow = ReactionType::try_from("➡️").expect("No right arrow");
		let save_icon = ReactionType::try_from("💾").expect("No floppy disk");
		let binder_icon = ReactionType::try_from(":pokeball:972277627077423124").expect("No pokeball");
		let mut player = player::get_player(msg.author.id.0).await;
		let embeds = self.iter().map(|e| e.embed()).collect::<Vec<_>>();
		let mut idx: i16 = 0;
		let mut content = String::from("");
		let mut message = msg
			.channel_id
			.send_message(&ctx.http, |m| {
				let mut cur_embed = embeds[idx as usize].clone();
				if embeds.len() > 1 {
					cur_embed.footer(|f| f.text(format!("{}/{}", idx + 1, embeds.len())));
				}
				let mut extra_desc = String::from("");
				if &player.current_binder.set == &self[idx as usize].set().id() {
					match player.current_binder.cards.contains(&self[idx as usize].card_id()) {
						true => extra_desc.push_str("<:pokeball:972277627077423124> In your binder\n"),
						false => extra_desc.push_str("<:GameCorner:967591653135228988> Not in your binder\n")
					}
				}
				if player.savelist.contains(&self[idx as usize].card_id()) {
					extra_desc.push_str(":white_check_mark: In your savelist");
				}
				cur_embed.description(format!("{}\n{}", &self[idx as usize].description(), extra_desc));
				m.set_embed(cur_embed);

				if embeds.len() > 1 {
					m.reactions([left_arrow.clone(), right_arrow.clone(), save_icon.clone(), binder_icon.clone()]);
				} else {
					m.reactions([save_icon.clone(), binder_icon.clone()]);
				}

				m			
			}).await.unwrap();
		
		loop {
			if embeds.len() <= 1 {
				break; // Exit before anything. Probably a way to do this before entering.
			}
			if let Some(reaction) = &message
				.await_reaction(&ctx)
				.timeout(StdDuration::from_secs(90))
				.author_id(msg.author.id)
				.removed(true)
				.await
			{
				let emoji = &reaction.as_inner_ref().emoji;
				match emoji.as_data().as_str() {
					"⬅️" => idx = (idx - 1).rem_euclid(embeds.len() as i16),
					"➡️" => idx = (idx + 1) % embeds.len() as i16,
					"💾" => {
						let card_id = &self[idx as usize].card_id();
						if player.savelist.clone().contains(&card_id) {
							let index = player.savelist.clone().iter().position(|c| c == card_id).unwrap();
							player.savelist.remove(index);
							content = format!("**{}** removed from your savelist!", &self[idx as usize].card_name());
						} else {
							player.savelist.push(card_id.clone());
							content = format!("**{}** added to your savelist!", &self[idx as usize].card_name());
						}
						player::update_player(&player, doc! { "$set": { "savelist": player.savelist.clone()}}).await;
					},
					"pokeball:972277627077423124" => {
						let card_id = self[idx as usize].card_id().clone();
						if player.current_binder.cards.contains(&card_id) {
							content = format!("**{}** is already in your binder!", &self[idx as usize].card_name());
						} else if &self[idx as usize].set().id() != &player.current_binder.set {
							content = String::from("This card doesn't go in your binder!");
						} else {
							let current_binder_set = sets::get_set(&player.current_binder.set).await.unwrap();
							let mut player_update = Document::new();
							*player.cards.entry(self[idx as usize].card_id()).or_insert(0) -= 1;
							if *player.cards.entry(self[idx as usize].card_id()).or_insert(0) == 0 {
								player.cards.remove(&card_id);
							}
							let mut player_cards = Document::new();
							for (crd, amt) in player.cards.iter() {
								player_cards.insert(crd, amt);
							}
							player_update.insert("cards", player_cards);
							player.current_binder.cards.push(self[idx as usize].card_id().clone());
							if player.current_binder.is_complete().await {
								player.completed_binders.push(player.current_binder.set);
								player.current_binder = binder::Binder::empty();
								player_update.insert("completed_binders", player.completed_binders.clone());
								content = format!("You completed the **{}** binder!", current_binder_set.name);
							} else {
								content = format!("You added **{}** to your binder!", &self[idx as usize].card_name());
							}
							player_update.insert("current_binder", player.current_binder.to_doc());
							let mut player_cards = Document::new();
							for (crd, amt) in player.cards.iter() {
								player_cards.insert(crd, amt);
							}
							player_update.insert("cards", player_cards);
							player::update_player(&player, doc! { "$set": player_update }).await;
						}
					}
					_ => {
						println!("{}", &emoji.as_data().as_str());
						continue
					}
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
				let mut extra_desc = String::from("");
				if &player.current_binder.set == &self[idx as usize].set().id() {
					match player.current_binder.cards.contains(&self[idx as usize].card_id()) {
						true => extra_desc.push_str("<:pokeball:972277627077423124> In your binder\n"),
						false => extra_desc.push_str("<:GameCorner:967591653135228988> Not in your binder\n")
					}
				}
				if player.savelist.contains(&self[idx as usize].card_id()) {
					extra_desc.push_str(":white_check_mark: In your savelist");
				}
				cur_embed.description(format!("{}\n{}", &self[idx as usize].description(), extra_desc));
				m.set_embed(cur_embed);
				m.content(content);

				m
			}).await.unwrap();

			content = String::from("");
		}

		Ok(())
	}
}

pub async fn player_cards(cards_hash: HashMap<String, i64>) -> Vec<PlayerCard> {
	let mut ret = vec![];
	let card_hash_clone = cards_hash.clone();
	let card_ids: Vec<String> = card_hash_clone.into_keys().collect();
	let cards = get_multiple_cards_by_id(card_ids.clone()).await;
	for card in cards {
		let amount = cards_hash.get(&card.id()).unwrap().to_owned();
		ret.push(PlayerCard {card, amount: amount.clone()});
	}

	ret
}