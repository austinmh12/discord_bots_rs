use std::{
	collections::HashMap,
};

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::{
	channel::{Channel, Message},
	gateway::Ready,
	misc::Mention,
	user::User,
};
use serenity::prelude::*;

#[command]
async fn amogus(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let mut amogus_alphabet = HashMap::new();
	amogus_alphabet.insert('a', "881569779637452811");
	amogus_alphabet.insert('b', "881573421161537546");
	amogus_alphabet.insert('c', "881570972367486976");
	amogus_alphabet.insert('d', "881573101635256351");
	amogus_alphabet.insert('e', "881565979233107980");
	amogus_alphabet.insert('f', "881573138348003339");
	amogus_alphabet.insert('g', "881569855311073371");
	amogus_alphabet.insert('h', "881565907820892230");
	amogus_alphabet.insert('i', "881570959188975636");
	amogus_alphabet.insert('j', "881573167435493436");
	amogus_alphabet.insert('k', "881570985948618782");
	amogus_alphabet.insert('l', "881573187400396870");
	amogus_alphabet.insert('m', "881569802567688212");
	amogus_alphabet.insert('n', "881573209596629002");
	amogus_alphabet.insert('o', "881569826479435806");
	amogus_alphabet.insert('p', "881573226789105665");
	amogus_alphabet.insert('q', "881573242979094528");
	amogus_alphabet.insert('r', "881570944160788561");
	amogus_alphabet.insert('s', "881565820097032322");
	amogus_alphabet.insert('t', "881573258988773417");
	amogus_alphabet.insert('u', "881569871194882108");
	amogus_alphabet.insert('v', "881573272481853512");
	amogus_alphabet.insert('w', "881573289921765456");
	amogus_alphabet.insert('x', "881573308833878066");
	amogus_alphabet.insert('y', "881573324348612629");
	amogus_alphabet.insert('z', "881573340903514195");

	let amogus_sentence: String = args.raw()
		.collect::<Vec<&str>>()
		.join(" ")
		.to_lowercase()
		.chars()
		.map(|x| match amogus_alphabet.get(&x) {
			Some(&y) => format!("<:amogus_{}:{}>", x, y),
			None => " ".to_string()
		}).collect();

	let _ = msg.delete(ctx).await;
	let _ = msg.channel_id.say(&ctx.http, amogus_sentence).await;

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

#[command]
async fn blue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let blue_sentence: String = args.raw()
		.collect::<Vec<&str>>()
		.join(" ")
		.to_lowercase()
		.chars()
		.map(|x| match x.is_alphabetic() {
			false => " ".to_string(),
			true => format!(":regional_indicator_{}:", x)
		}).collect();

	let _ = msg.delete(ctx).await;
	let _ = msg.channel_id.say(&ctx.http, blue_sentence).await;

	Ok(())
}