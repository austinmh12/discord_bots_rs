use dotenv;

use serenity::{
	async_trait,
	model::{channel::Message, gateway::Ready},
	prelude::*
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	// Set a handler for the `message` event so that whenever a new message is received the closure (or function) passed will be called
	// Event handlers are dispatched through a threadpool, and so multiple events can be dispatched simultaneously.
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.content == "!ping" {
			// Sending a message can fail due to a network error, auth error, or permission error.
			if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
				println!("Error sending message: {:?}", why);
			}
		}
	}

	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();
	// Configure the client with the discord token
	let token = dotenv::var("AUSTINTOKEN").expect("Expected a token in the environment");
	println!("{}", token);

	// Create a new instance of the client logging in as the bot. This will automatically
	// prepend your bot token with "Bot ", which is required by discord.
	let mut client = Client::builder(&token).event_handler(Handler).await.expect("Error creating client");

	// Finally start a shard and listen for events.

	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}