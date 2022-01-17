mod commands;

use dotenv;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler, bridge::gateway::GatewayIntents};
use serenity::model::{
	channel::{Channel, Message, Reaction, ReactionType},
	gateway::Ready,
	misc::Mention,
	user::User,
	id::{EmojiId}
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

use commands::{meme::*};

#[group]
#[commands(sheesh, amogus, blue)]
struct Meme;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	// Set the handler to be called on the `ready` event. This is called when a shard is booted, and a READY payload is sent by Discord.
	// This payload contains a bunch of data.
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}

	async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
		if reaction.emoji.unicode_eq("✅") {
			let _ = reaction.message(&ctx).await.unwrap()
				.react(&ctx, ReactionType::try_from("<:backchk:799333634263613440>").unwrap()).await;
		}
	}

	// async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {
	// 	if _add_reaction.emoji.unicode_eq("\u{2705}") {
	// 		let msg = _add_reaction.message(_ctx).await.unwrap();
	// 		let _ = msg.react(_ctx.http, EmojiId::from(799333634263613440));
	// 	}
	// }
}

#[tokio::main]
async fn main() {
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("."))
		.group(&MEME_GROUP);

	dotenv::dotenv().ok();
	// Configure the client with the discord token
	let token = dotenv::var("TESTBOT").expect("Expected a token in the environment");

	// Create a new instance of the client logging in as the bot. This will automatically
	// prepend your bot token with "Bot ", which is required by discord.
	let mut client = Client::builder(&token)
		.event_handler(Handler)
		.intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS | GatewayIntents::default())
		.framework(framework)
		.await
		.expect("Error creating client");

	// Finally start a shard and listen for events.
	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}