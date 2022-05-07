use std::io::{Cursor};

use image::{io::Reader, DynamicImage};
use mongodb::{
	bson::{
		doc,
		oid::ObjectId,
	}, 
	Collection
};
use serde::{Serialize, Deserialize};

use crate::commands::get_client;

use super::card::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardImage {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id: Option<ObjectId>,
	pub card_id: String,
	pub data: String
}

impl CardImage {
	pub fn to_dyn_image(&self) -> DynamicImage {
		let data = base64::decode(&self.data).unwrap();
		let img = image::load_from_memory(&data).unwrap();
		
		img
	}
}


async fn get_card_image_collection() -> Collection<CardImage> {
	let client = get_client().await.unwrap();
	let collection = client.database("poketcg").collection::<CardImage>("greyscale");

	collection
}



pub async fn get_card_image(card: &Card) -> CardImage {
	let card_image_collection = get_card_image_collection().await;
	let card_image = card_image_collection
		.find_one(doc! { "card_id": &card.card_id }, None)
		.await
		.unwrap();
	match card_image {
		Some(x) => return x,
		None => return add_card_image(card).await,
	}
}

async fn add_card_image(card: &Card) -> CardImage {
	let resp = reqwest::Client::new()
		.get(&card.image)
		.send().await.unwrap()
		.bytes().await.unwrap();
	let reader = Reader::new(Cursor::new(resp))
		.with_guessed_format()
		.expect("Can't get image");
	let image = reader.decode().unwrap().grayscale();
	let mut buf: Vec<u8> = vec![];
	let mut bw = Cursor::new(&mut buf);
	image.write_to(&mut bw, image::ImageOutputFormat::Png).unwrap();
	let img_b64 = base64::encode(&buf);
	let card_image = CardImage{
		id: None,
		card_id: String::from(&card.card_id),
		data: img_b64
	};
	let card_image_collection = get_card_image_collection().await;
	card_image_collection
	.insert_one(card_image.clone(), None)
		.await
		.unwrap();

	card_image
}