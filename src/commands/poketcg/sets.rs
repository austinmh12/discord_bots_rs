use super::*;
use chrono::{
	NaiveDate,
	DateTime,
	Utc, Datelike,
};

#[derive(Clone, Debug)]
pub struct Set {
	pub id: String,
	pub name: String,
	pub series: String,
	pub printed: i32,
	pub total: i32,
	pub logo: String,
	pub symbol: String,
	pub release_date: DateTime<Utc>
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
			release_date: DateTime::<Utc>::from_utc(
				NaiveDate::parse_from_str(obj["releaseDate"].as_str().unwrap(), "%Y/%m/%d")
					.unwrap()
					.and_hms(0, 0, 0),
				Utc
			)
		}
	}
	
	pub fn pack_price(&self) -> f64 {
		let now = Utc::now();
		let date_power = now.year() - &self.release_date.year();

		((3.75 * 1.1_f64.powi(date_power)) * 100.0).round() / 100.0
	}
}

impl PaginateEmbed for Set {
	fn embed(&self) -> CreateEmbed {
		let mut ret = CreateEmbed::default();
		ret
			.title(&self.name)
			.description(format!("**Series:** {}\n**Total cards:** ${}\n**Pack price:** ${:.2}\n**ID:** {}", &self.series, &self.printed, &self.pack_price(), &self.id))
			.colour(Colour::from_rgb(255, 50, 20))
			.image(&self.logo)
			.thumbnail(&self.symbol);

		ret
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

pub async fn get_set(id: &str) -> Option<Set> {
	let data = api_call(&format!("sets/{}", id), None)
		.await
		.unwrap();
	let set_data = data.get("data");
	let set = match set_data {
		Some(x) => Some(Set::from_json(x)),
		None => None
	};

	set
}

pub async fn get_sets_with_query(query: &str) -> Vec<Set> {
	let mut ret = vec![];
	let data = api_call("sets", Some(query)).await.unwrap();
	let set_data = data["data"].as_array().unwrap();
	for st in set_data {
		let set = Set::from_json(st);
		ret.push(set);
	}

	ret
}