use std::{
	collections::HashMap,
	time::Duration,
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
	prelude::*,
};
use serenity::utils::Colour;
use serenity::prelude::*;
use serenity::collector::MessageCollectorBuilder;
use serde_json;

struct YouTubeSearchResult {
	pub channel_id: String,
	pub title: String,
	pub thumbnail: String
}

impl YouTubeSearchResult {
	pub fn new(channel_id: String, title: String, thumbnail: String) -> Self {
		Self {
			channel_id,
			title,
			thumbnail
		}
	}
}

struct YouTubeChannel {
	pub channel_id: String,
	pub title: String,
	pub thumbnail: String,
	pub video_count: i32
}

impl YouTubeChannel {
	pub fn new(channel_id: String, title: String, thumbnail: String, video_count: i32) -> Self {
		Self {
			channel_id,
			title,
			thumbnail,
			video_count
		}
	}

	pub fn from_search(search_result: YouTubeSearchResult) -> Self {
		let channel_id = search_result.channel_id;
		let title = search_result.title;
		let thumbnail = search_result.thumbnail;
		let video_count = 0;
		
		Self {
			channel_id,
			title,
			thumbnail,
			video_count
		}
	}
}

struct Subscription {
	pub discord_id: i32,
	pub channel_id: String
}

impl Subscription {
	pub fn new(discord_id: i32, channel_id: String) -> Self {
		Self {
			discord_id,
			channel_id
		}
	}
}

// Database queries
async fn get_database_connection() -> sqlx::Pool<sqlx::Sqlite> {
	let database = sqlx::sqlite::SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(
			sqlx::sqlite::SqliteConnectOptions::new()
				.filename("sqlite:///bot.db")
				.create_if_missing(true),
		)
		.await
		.expect("Couldn't connect to database");
	
	database
}

async fn get_channels() -> Vec<YouTubeChannel> {
	let database = get_database_connection().await;
	let channels = sqlx::query_as!(YouTubeChannel, "select * from channels;")
		.fetch_all(&database)
		.await
		.unwrap();

	channels
}

async fn get_channel(channel_id: String) -> YouTubeChannel {
	let database = get_database_connection().await;
	let channel = sqlx::query_as!(YouTubeChannel, "select * from channels where channel_id = ?", channel_id)
		.fetch_one(&database)
		.await
		.unwrap();

	channel
}

async fn add_channel(channel: YouTubeChannel) {
	let database = get_database_connection().await;
	sqlx::query!(
		"insert into channels values (?,?,?,?)",
		channel.channel_id,
		channel.title,
		channel.thumbnail,
		channel.video_count
	)
		.execute(&database)
		.await
		.unwrap();
}

async fn delete_channel(channel: YouTubeChannel) {
	let database = get_database_connection().await;
	sqlx::query!("delete from channels where channel_id = ?", channel.channel_id)
		.execute(&database)
		.await
		.unwrap();
}

async fn update_channel(channel: YouTubeChannel) {
	let database = get_database_connection().await;
	sqlx::query!("update channels set video_count = ? where channel_id = ?", channel.video_count, channel.channel_id)
		.execute(&database)
		.await
		.unwrap();
}

async fn get_subscriptions_for_user(discord_id: i32) -> Vec<Subscription> {
	let database = get_database_connection().await;
	let subs = sqlx::query_as!(Subscription, "select * from subscriptions where discord_id = ?", discord_id)
		.fetch_all(&database)
		.await
		.unwrap();

	subs
}

// Utilities for commands
async fn search_youtube(search: &str) -> Result<Vec<YouTubeSearchResult>, reqwest::Error> {
	dotenv::dotenv().ok();
	let youtube_api_key = dotenv::var("YTAPIKEY").unwrap();
	let mut resp: serde_json::Value = reqwest::Client::new()
		.get(format!("https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&type=channel&key={}", search, youtube_api_key))
		.send()
		.await?
		.json()
		.await?;
	let search_results = resp["items"].as_array_mut().unwrap();
	let mut channels_searched = <Vec<YouTubeSearchResult>>::new();
	for search_result in search_results {
		let snippet = search_result["snippet"].as_object_mut().unwrap();
		channels_searched.push(YouTubeSearchResult::new(
			snippet["channelId"].as_str().unwrap().to_string(),
			snippet["title"].as_str().unwrap().to_string(),
			snippet["thumbnails"]["default"]["url"].as_str().unwrap().to_string()
		));
	}

	Ok(channels_searched)
}


#[command]
#[aliases(sub)]
#[min_args(1)]
async fn subscribe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let search_string = args.rest().replace(" ", "%20"); // Converts .subscribe linus tech tips to linus%20tech%20%tips
	let channels_searched = search_youtube(&search_string).await?;
	let mut desc = String::from("Here are the results of your search, reply with a number to make a selection.\n");
	for (i, channel) in channels_searched.iter().enumerate() {
		desc.push_str(
			&format!("**{}:** [{}]({})\n", i + 1, channel.title, format!("https://www.youtube.com/channel/{}", channel.channel_id))
		);
	}
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title("Search Results")
					.description(desc)
					.thumbnail(channels_searched[0].thumbnail.clone())
					.colour(Colour::from_rgb(255, 50, 20))
			})
		})
		.await;

	let selection_range = 1..5;
	if let Some(reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		let user_selection = reply.content.parse::<i32>().unwrap();
		if selection_range.contains(&user_selection) {
			// check for existing subscription
			let _ = msg
				.channel_id
				.say(
					&ctx.http,
					format!("You subscribed to **{}**", channels_searched[(user_selection - 1) as usize].title)
				)
				.await;
				// Do database stuff to add the subscription
				// Do database stuff to add channel
		} else {
			let _ = msg.channel_id.say(&ctx.http, format!("{} was not a valid selection", user_selection)).await;
		}
	} else {
		let _ = msg.channel_id.say(&ctx.http, "A selection was not made.").await;
	};

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
}

// Background task to loop through set of YouTubeChannels and fetch video counts with reqwest