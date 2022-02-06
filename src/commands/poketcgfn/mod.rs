use dotenv;
use serde_json;

fn api_call(endpoint: String, params: Vec<Tuple<String, String>>) -> serde_json::Value {
	dotenv::dotenv().ok();
	let poketcg_key = dotenv::var("POKETCGAPIKEY").unwrap();
	return reqwest::Client::new()
		.get(format!("https://api.pokemontcg.io/v2/{}", endpoint))
		.header("X-Api-Key", poketcg_key)
		.query(&params)
		.send()?
		.json()?;
}

pub mod card;