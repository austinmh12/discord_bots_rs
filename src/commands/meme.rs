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

lazy_static! {
	static ref AMOGUS_ALPHABET: HashMap<char, &'static str> = {
		let mut m = HashMap::new();
		m.insert('a', "881569779637452811");
		m.insert('b', "881573421161537546");
		m.insert('c', "881570972367486976");
		m.insert('d', "881573101635256351");
		m.insert('e', "881565979233107980");
		m.insert('f', "881573138348003339");
		m.insert('g', "881569855311073371");
		m.insert('h', "881565907820892230");
		m.insert('i', "881570959188975636");
		m.insert('j', "881573167435493436");
		m.insert('k', "881570985948618782");
		m.insert('l', "881573187400396870");
		m.insert('m', "881569802567688212");
		m.insert('n', "881573209596629002");
		m.insert('o', "881569826479435806");
		m.insert('p', "881573226789105665");
		m.insert('q', "881573242979094528");
		m.insert('r', "881570944160788561");
		m.insert('s', "881565820097032322");
		m.insert('t', "881573258988773417");
		m.insert('u', "881569871194882108");
		m.insert('v', "881573272481853512");
		m.insert('w', "881573289921765456");
		m.insert('x', "881573308833878066");
		m.insert('y', "881573324348612629");
		m.insert('z', "881573340903514195");
		m
	};
}

#[command]
async fn amogus(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let amogus_sentence: String = args.raw()
		.collect::<Vec<&str>>()
		.join(" ")
		.to_lowercase()
		.chars()
		.map(|x| match AMOGUS_ALPHABET.get(&x) {
			Some(&y) => format!("<:amogus_{}:{}>", x, y),
			None => " ".to_string()
		}).collect();

	let _ = msg.delete(ctx).await;
	let _ = msg.channel_id.say(&ctx.http, amogus_sentence).await;

	Ok(())
}

#[command]
#[max_args(2)]
async fn sheesh(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let num_e_range = 4..3900;
	let _ = msg.delete(ctx).await;
	let nickname = msg.author_nick(ctx).await.unwrap();
	
	// Find is better for typed optional arguments.
	let num_es = match args.find::<i32>() {
		Ok(n) => if num_e_range.contains(&n) {
			n
		} else {
			4
		},
		Err(_) => 4
	};
	let mention = match args.find::<String>() {
		Ok(m) => m,
		Err(_) => "".to_string()
	};
	let es = (0..num_es).map(|_| "e").collect::<String>(); 
	let content = if mention == "" {
		format!("***SHEEE{}***eeesh\n> - _{}_", es, nickname)
	} else {
		format!("***SHEEE{}***eeesh {}\n> - _{}_", es, mention, nickname)
	};

	let _ = msg.channel_id.say(&ctx.http, content).await;

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