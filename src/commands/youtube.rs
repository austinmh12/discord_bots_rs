use std::{
	collections::HashMap,
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
};
use serenity::prelude::*;

struct YouTubeChannel {
	pub id: String
	pub url: String
	pub name: String
	pub thumbnail: String
	// pub colour: // Idk at the moment
	pub video_count: i32
	pub upload_id: String
	priv latest_video: String
}

#[command]
#[min_args(1)]
async fn subscribe(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Implement this

	Ok(())
}

#[command]
#[min_args(1)]
async fn unsubscribe(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Implement this

	Ok(())
}

#[command]
async fn subscriptions(ctx: &Context, msg: &Message) -> CommandResult {
	// TODO: Implement this

	Ok(())
})

// Background task to loop through set of YouTubeChannels and fetch video counts with reqwest