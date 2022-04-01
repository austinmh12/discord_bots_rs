use super::*;

pub struct Set {
	pub id: String,
	pub name: String,
	pub series: String,
	pub printed: i32,
	pub total: i32,
	pub logo: String,
	pub symbol: String,
	pub releaseDate: String // Will be a datetime eventually?
}

impl Set {
	pub fn from_json(obj: &serde_json::Value) -> Self {
		Self {
			id: String::from(obj["id"].unwrap()),
			name: String::from(obj["name"].unwrap()),
			series: String::from(obj["series"].unwrap()),
			printed: obj["printedTotal"].unwrap().parse::<i32>(),
			total: obj["total"].unwrap().parse::<i32>(),
			logo: String::from(obj["images"]["logo"].unwrap()),
			symbol: String::from(obj["images"]["symbol"].unwrap()),
			releaseDate: String::from(obj["releaseDate"].unwrap())
		}
	}
}

pub fn get_sets() -> Vec<Set> {
	let mut ret = vec![];

	ret
}

pub fn get_set(id: &String) -> Set {
	
}