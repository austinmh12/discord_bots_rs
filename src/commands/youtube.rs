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

impl YouTubeSearchResult {
	pub fn new(channel_id: String, title: String, thumbnail: String) -> Self {
		Self {
			channel_id,
			title,
			thumbnail
		}
	}
}

#[derive(Clone)]
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

	pub fn from_search(search_result: &YouTubeSearchResult) -> Self {
		let channel_id = search_result.channel_id.as_str().to_string();
		let title = search_result.title.as_str().to_string();
		let thumbnail = search_result.thumbnail.as_str().to_string();
		let video_count = 0;
		
		Self {
			channel_id,
			title,
			thumbnail,
			video_count
		}
	}

	pub async fn get_upload_id(&self) -> String {
		dotenv::dotenv().ok();
		let youtube_api_key = dotenv::var("YTAPIKEY").unwrap();
		let resp: serde_json::Value = reqwest::Client::new()
			.get(format!("https://www.googleapis.com/youtube/v3/channels?key={}&id={}&part=contentDetails", youtube_api_key, self.channel_id))
			.send().await.unwrap()
			.json().await.unwrap();
		let upload_id = resp["items"][0]["contentDetails"]["relatedPlaylists"]["uploads"].as_str().unwrap();

		String::from(upload_id)
	}

	pub async fn get_video_count(&self) -> i32 {
		dotenv::dotenv().ok();
		let youtube_api_key = dotenv::var("YTAPIKEY").unwrap();
		let resp: serde_json::Value = reqwest::Client::new()
			.get(format!("https://www.googleapis.com/youtube/v3/playlistItems?key={}&part=contentDetails&playlistId={}", youtube_api_key, self.get_upload_id().await))
			.send().await.unwrap()
			.json().await.unwrap();
		let video_count = resp["pageInfo"]["totalResults"].as_i64().unwrap();

		video_count as i32
	}

	pub async fn get_latest_video(&self) -> Video {
		dotenv::dotenv().ok();
		let youtube_api_key = dotenv::var("YTAPIKEY").unwrap();
		let resp: serde_json::Value = reqwest::Client::new()
			.get(format!("https://www.googleapis.com/youtube/v3/playlistItems?key={}&part=contentDetails,snippet&playlistId={}", youtube_api_key, self.get_upload_id().await))
			.send().await.unwrap()
			.json().await.unwrap();
		let video = &resp["items"][0];
		let thumbnails = video["snippet"]["thumbnails"].as_object().unwrap();
		let thumbnail = match thumbnails.get("standard") {
			Some(x) => String::from(x["url"].as_str().unwrap()),
			None => String::from(thumbnails["default"]["url"].as_str().unwrap())
		};

		Video::new(
			String::from(video["contentDetails"]["videoId"].as_str().unwrap()),
			String::from(video["snippet"]["title"].as_str().unwrap()),
			String::from(video["snippet"]["description"].as_str().unwrap()),
			thumbnail,
			String::from(video["contentDetails"]["videoPublishedAt"].as_str().unwrap()),
			self.clone()
		)
	}
}

struct Subscription {
	pub discord_id: String,
	pub channel_id: String
}

impl Subscription {
	pub fn new(discord_id: String, channel_id: String) -> Self {
		Self {
			discord_id,
			channel_id
		}
	}
}

struct Video {
	pub video_id: String,
	pub title: String,
	pub description: String,
	pub thumbnail: String, 
	pub uploaded: String, // TODO: Make this a datetime for formatting reasons
	pub channel: YouTubeChannel
}

impl Video {
	pub fn new(video_id: String, title: String, description: String, thumbnail: String, uploaded: String, channel: YouTubeChannel) -> Self {
		Self {
			video_id,
			title,
			description,
			thumbnail,
			uploaded,
			channel
		}
	}
}

// Database queries
async fn get_database_connection() -> sqlx::Pool<sqlx::Sqlite> {
	let database = sqlx::sqlite::SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(
			sqlx::sqlite::SqliteConnectOptions::new()
				.filename("bot.db")
				.create_if_missing(true),
		)
		.await
		.expect("Couldn't connect to database");
	
	database
}

async fn get_channels() -> Vec<YouTubeChannel> {
	let database = get_database_connection().await;
	let channels = sqlx::query_as!(YouTubeChannel, r#"select channel_id, title, thumbnail, video_count as "video_count: i32" from channels;"#)
		.fetch_all(&database)
		.await
		.unwrap();

	channels
}

async fn get_channel(channel_id: String) -> YouTubeChannel {
	let database = get_database_connection().await;
	let channel = sqlx::query_as!(YouTubeChannel, r#"select channel_id, title, thumbnail, video_count as "video_count: i32" from channels where channel_id = ?;"#, channel_id)
		.fetch_one(&database)
		.await
		.unwrap();

	channel
}

async fn add_channel(channel: YouTubeChannel) {
	// TODO: Add a check to not add duplicates.
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

async fn get_subscriptions_for_user(discord_id: String) -> Vec<Subscription> {
	let database = get_database_connection().await;
	let subs = sqlx::query_as!(Subscription, "select * from subscriptions where discord_id = ?", discord_id)
		.fetch_all(&database)
		.await
		.unwrap();

	subs
}

async fn get_subscriptions_for_channel(channel: YouTubeChannel) -> Vec<Subscription> {
	let database = get_database_connection().await;
	let subs = sqlx::query_as!(Subscription, "select * from subscriptions where channel_id = ?", channel.channel_id)
		.fetch_all(&database)
		.await
		.unwrap();

	subs
}

async fn add_subscription(sub: Subscription) {
	let database = get_database_connection().await;
	sqlx::query!("insert into subscriptions values (?,?)", sub.discord_id, sub.channel_id)
		.execute(&database)
		.await
		.unwrap();
}

async fn delete_subscription(sub: Subscription) {
	let database = get_database_connection().await;
	sqlx::query!("delete from subscriptions where discord_id = ? and channel_id = ?", sub.discord_id, sub.channel_id)
		.execute(&database)
		.await
		.unwrap();
}

async fn check_for_existing_sub(discord_id: String, channel: &YouTubeSearchResult) -> bool {
	let user_subs = get_subscriptions_for_user(discord_id).await;
	let sub_in_subs = user_subs.iter().any(|us| {
		us.channel_id == channel.channel_id
	});

	sub_in_subs
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
			String::from(snippet["channelId"].as_str().unwrap()),
			String::from(snippet["title"].as_str().unwrap()),
			String::from(snippet["thumbnails"]["default"]["url"].as_str().unwrap())
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

	let selection_range = 1..6;
	if let Some(reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		let user_selection = reply.content.parse::<i32>().unwrap();
		if selection_range.contains(&user_selection) {
			let channel = &channels_searched[(user_selection - 1) as usize];
			if check_for_existing_sub(format!("{}", msg.author.id.0), &channel).await {
				let _ = msg.channel_id.say(&ctx.http, format!("You are already subscribed to **{}**", channel.title)).await;
			} else {
				let _ = msg.channel_id.say(&ctx.http, format!("You subscribed to **{}**", channel.title)).await;
				let channel_id = &channel.channel_id;
				add_channel(YouTubeChannel::from_search(channel)).await;
				add_subscription(Subscription {
					discord_id: format!("{}", msg.author.id.0),
					channel_id: String::from(channel_id)
				}).await;
			}
		} else {
			let _ = msg.channel_id.say(&ctx.http, format!("{} was not a valid selection", user_selection)).await;
		}
	} else {
		let _ = msg.channel_id.say(&ctx.http, "A selection was not made.").await;
	};

	Ok(())
}

#[command]
#[aliases(unsub)]
async fn unsubscribe(ctx: &Context, msg: &Message) -> CommandResult {
	let subs = get_subscriptions_for_user(format!("{}", msg.author.id.0)).await;
	let mut sub_channels: Vec<YouTubeChannel> = <Vec<YouTubeChannel>>::new();
	for sub in subs {
		let channel = get_channel(sub.channel_id).await;
		sub_channels.push(channel);
	}

	sub_channels.sort_by(|a, b| a.title.cmp(&b.title));

	let nickname = msg.author_nick(ctx).await.unwrap();
	let mut desc = String::from("Which channel would you like to unsubscribe from? (type the number)\n");
	for (i, channel) in sub_channels.iter().enumerate() {
		desc.push_str(
			&format!("**{}:** [{}]({})\n", i + 1, channel.title, format!("https://www.youtube.com/channel/{}", channel.channel_id))
		);
	}
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(format!("{}'s Subscriptions\n", nickname))
					.description(desc)
					.thumbnail(sub_channels[0].thumbnail.clone())
					.colour(Colour::from_rgb(255, 50, 20))
			})
		})
		.await;

	let selection_range = 1..sub_channels.len() as i32 + 1;
	if let Some(reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		let user_selection = reply.content.parse::<i32>().unwrap();
		if selection_range.contains(&user_selection) {
			let channel = &sub_channels[(user_selection - 1) as usize];
			let _ = msg.channel_id.say(&ctx.http, format!("You unsubscribed from **{}**", channel.title)).await;
			let channel_id = &channel.channel_id;
			delete_subscription(Subscription::new(format!("{}", msg.author.id.0), String::from(channel_id))).await;
			if get_subscriptions_for_channel(channel.clone()).await.len() == 0 {
				delete_channel(channel.clone()).await;
			}
		} else {
			let _ = msg.channel_id.say(&ctx.http, format!("{} was not a valid selection", user_selection)).await;
		}
	} else {
		let _ = msg.channel_id.say(&ctx.http, "A selection was not made.").await;
	};

	Ok(())
}

#[command]
#[aliases(subs)]
async fn subscriptions(ctx: &Context, msg: &Message) -> CommandResult {
	let subs = get_subscriptions_for_user(format!("{}", msg.author.id.0)).await;
	let mut sub_channels: Vec<YouTubeChannel> = <Vec<YouTubeChannel>>::new();
	for sub in subs {
		let channel = get_channel(sub.channel_id).await;
		sub_channels.push(channel);
	}

	sub_channels.sort_by(|a, b| a.title.cmp(&b.title));

	let nickname = msg.author_nick(ctx).await.unwrap();
	let mut desc = String::from("");
	for (i, channel) in sub_channels.iter().enumerate() {
		desc.push_str(
			&format!("[{}]({})\n", channel.title, format!("https://www.youtube.com/channel/{}", channel.channel_id))
		);
	}
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(format!("{}'s Subscriptions\n", nickname))
					.description(desc)
					.thumbnail(sub_channels[0].thumbnail.clone())
					.colour(Colour::from_rgb(255, 50, 20))
			})
		})
		.await;

	Ok(())
}

#[command]
#[aliases(latest)]
async fn latest_video(ctx: &Context, msg: &Message) -> CommandResult {
	let subs = get_subscriptions_for_user(format!("{}", msg.author.id.0)).await;
	let mut sub_channels: Vec<YouTubeChannel> = <Vec<YouTubeChannel>>::new();
	for sub in subs {
		let channel = get_channel(sub.channel_id).await;
		sub_channels.push(channel);
	}

	sub_channels.sort_by(|a, b| a.title.cmp(&b.title));

	let nickname = msg.author_nick(ctx).await.unwrap();
	let mut desc = String::from("What channel would you like to get the latest video for? (type the number)\n");
	for (i, channel) in sub_channels.iter().enumerate() {
		desc.push_str(
			&format!("**{}:** [{}]({})\n", i + 1, channel.title, format!("https://www.youtube.com/channel/{}", channel.channel_id))
		);
	}
	let _ = msg
		.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(format!("{}'s Subscriptions\n", nickname))
					.description(desc)
					.thumbnail(sub_channels[0].thumbnail.clone())
					.colour(Colour::from_rgb(255, 50, 20))
			})
		})
		.await;

	let selection_range = 1..sub_channels.len() as i32 + 1;
	if let Some(reply) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(30)).await {
		let user_selection = reply.content.parse::<i32>().unwrap();
		if selection_range.contains(&user_selection) {
			let channel = &sub_channels[(user_selection - 1) as usize];
			let video = channel.get_latest_video().await;
			let _ = msg
				.channel_id
				.send_message(&ctx.http, |m| {
					m.embed(|e| {
						e.title(video.title)
							.description(format!("[Watch here!](https://www.youtube.com/watch?v={})\n\n{}", video.video_id, video.description))
							.thumbnail(video.channel.thumbnail)
							.colour(Colour::from_rgb(255, 50, 20))
							.image(video.thumbnail)
					})
				})
				.await;
			
		} else {
			let _ = msg.channel_id.say(&ctx.http, format!("{} was not a valid selection", user_selection)).await;
		}
	} else {
		let _ = msg.channel_id.say(&ctx.http, "A selection was not made.").await;
	};

	Ok(())
}

// TODO: Make the descriptions of videos split on the first \n if there is one
// TODO: Make the new video thing check join all the discord Id's of the subscribers for the 
	// New **channel.title** video @user, @user

// Background task to loop through set of YouTubeChannels and fetch video counts with reqwest
pub async fn check_for_new_videos(ctx: Arc<Context>) {
	// let discord_channel = ChannelId(623291442726436884); // Youtuber Updates
	let discord_channel = ChannelId(932619188323885076); // testing-bot
	let channels = get_channels().await;
	for mut channel in channels {
		let channel_subs = get_subscriptions_for_channel(channel.clone()).await;
		if channel_subs.len() == 0 {
			delete_channel(channel.clone()).await;
			continue;
		}

		let video_count = channel.get_video_count().await;
		if video_count > channel.clone().video_count {
			channel.video_count = video_count;
			update_channel(channel.clone()).await;
			let video = channel.get_latest_video().await;
			let _ = discord_channel
				.send_message(&ctx.http, |m| {
					m.embed(|e| {
						e.title(video.title)
							.description(format!("[Watch here!](https://www.youtube.com/watch?v={})\n\n{}", video.video_id, video.description))
							.thumbnail(video.channel.thumbnail)
							.colour(Colour::from_rgb(255, 50, 20))
							.image(video.thumbnail)
					})
				})
				.await;
		}
	}
}