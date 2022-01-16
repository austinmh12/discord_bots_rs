use dotenv;

use std::fmt;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler, bridge::gateway::GatewayIntents};
use serenity::model::{
	channel::{Channel, Message},
	gateway::Ready,
	misc::Mention,
	user::User,
};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group,
    },
	Args,
	Delimiter,
};

#[group]
#[commands(ping, sheesh)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("."))
		.group(&GENERAL_GROUP);

	dotenv::dotenv().ok();
	// Configure the client with the discord token
	let token = dotenv::var("AUSTINTOKEN").expect("Expected a token in the environment");

	// Create a new instance of the client logging in as the bot. This will automatically
	// prepend your bot token with "Bot ", which is required by discord.
	let mut client = Client::builder(&token)
		.event_handler(Handler)
		.framework(framework)
		.await
		.expect("Error creating client");

	// Finally start a shard and listen for events.
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.reply(ctx, "Pong!").await?;

	Ok(())
}

#[command]
async fn sheesh(ctx: &Context, msg: &Message) -> CommandResult {
	// let mut num_es = args.single::<i32>().unwrap();
	// let mention = args.single::<Mention>().unwrap();

	let _ = msg.delete(ctx).await;
	
	let nickname = msg.author_nick(ctx).await;
	let _ = msg.channel_id.say(&ctx.http, format!("***SHEEEeeee***eeesh\n> - _{}_", nickname.unwrap())).await;

	Ok(())
}