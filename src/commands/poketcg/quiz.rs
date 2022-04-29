use rand::{
	prelude::*
};
use image::{
	io::Reader,
	Rgba
};
use std::io::Cursor;

const NAME_TO_KEEP_DASH: &'static [&str] = &[
	"ho-oh",
	"porygon-z",
	"type-null",
	"jangmo-o",
	"hakamo-o",
	"kommo-o"
];
const NAMES_TO_REPLACE_DASH: &'static [&str] = &[
	"mr-mime",
	"mr-rime",
	"mime-jr",
	"tapu-koko",
	"tapu-lele",
	"tapu-bulu",
	"tapu-fini"
];
pub struct Quiz {
	pub national_id: i64,
	pub name: String,
	pub generation: i64
}

impl Quiz {
	pub async fn random_quiz() -> Self {
		let national_id = rand::thread_rng().gen_range(1..=898) as i64;
		let resp: serde_json::Value = reqwest::Client::new()
			.get(format!("https://pokeapi.co/api/v2/pokemon/{}", &national_id))
			.send().await.unwrap()
			.json().await.unwrap();
		let name = String::from(resp["name"].as_str().unwrap());
		let generation = match national_id {
			1..=151 => 1,
			152..=251 => 2,
			252..=386 => 3,
			387..=493 => 4,
			494..=649 => 5,
			650..=721 => 6,
			722..=809 => 7,
			_ => 8
		};

		Self {
			national_id,
			name,
			generation
		}
	}

	pub async fn generate_silhouette(&self) {
		let resp = reqwest::Client::new()
			.get(format!("https://img.pokemondb.net/sprites/home/normal/{}.png", self.name))
			.send().await.unwrap()
			.bytes().await.unwrap();
		let reader = Reader::new(Cursor::new(resp))
			.with_guessed_format()
			.expect("Can't get image");
		let mut image = reader.decode().unwrap().into_rgba8();
		image.save("quizresult.PNG").unwrap();
		let (width, height) = image.dimensions();
		for i in 0..width {
			for j in 0..height {
				let px = image.get_pixel(i, j);
				if px.0[3] != 0 {
					image.put_pixel(i, j, Rgba([0, 0, 0, 255]));
				}
			}
		}
		image.save("quizsilhouette.PNG").unwrap();
	}

	pub fn guess_name(&self) -> String {
		if NAME_TO_KEEP_DASH.contains(&self.name.as_str()) {
			return self.name.clone();
		}
		let mut name = self.name.clone();
		if NAMES_TO_REPLACE_DASH.contains(&name.as_str()) {
			name = name.replace("-", " ").to_string();
		}
		if name.contains("-") {
			name = name.split("-").collect::<Vec<&str>>()[0].to_string();
		}

		name
	}
}