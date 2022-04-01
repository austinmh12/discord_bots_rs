use super::*;

pub struct Set {
	pub id: String,
	pub name: String,
	pub series: String,
	pub printed: i32,
	pub total: i32,
	pub logo: String,
	pub symbol: String,
	pub release_date: String // Will be a datetime eventually?
}

impl Set {
	pub fn from_json(obj: &serde_json::Value) -> Self {
		Self {
			id: String::from(obj["id"].as_str().unwrap()),
			name: String::from(obj["name"].as_str().unwrap()),
			series: String::from(obj["series"].as_str().unwrap()),
			printed: obj["printedTotal"].as_i64().unwrap() as i32,
			total: obj["total"].as_i64().unwrap() as i32,
			logo: String::from(obj["images"]["logo"].as_str().unwrap()),
			symbol: String::from(obj["images"]["symbol"].as_str().unwrap()),
			release_date: String::from(obj["releaseDate"].as_str().unwrap())
		}
	}
}

pub async fn get_sets() -> Vec<Set> {
	let mut ret = vec![];
	let data = api_call("sets", None)
		.await
		.unwrap();
	let set_data = data["data"].as_array().unwrap();
	for s in set_data {
		let set = Set::from_json(s);
		ret.push(set);
	}

	ret
}

pub fn get_set(id: &String) -> Set {
	unimplemented!();
}