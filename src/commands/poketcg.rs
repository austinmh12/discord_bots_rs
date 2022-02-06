use std::{
	time::Duration,
	sync::{
		Arc,
	}
};

use serenity::framework::standard::{
	macros::{
		command,
	},
	Args,
	CommandResult
};
use serenity::model::{
	channel::Message,
	id::{ChannelId}
	//prelude::*,
};
use serenity::utils::Colour;
use serenity::prelude::*;
//use serenity::collector::MessageCollectorBuilder;
use serde_json;

struct YouTubeSearchResult {
	pub channel_id: String,
	pub title: String,
	pub thumbnail: String
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