pub mod meme;
// pub mod youtube;
pub mod poketcg;

use mongodb::{
	Client,
	options::{
		ClientOptions,
	},
};
use std::error::Error;

// Database queries
// async fn get_database_connection() -> sqlx::Pool<sqlx::Sqlite> {
// 	let database = sqlx::sqlite::SqlitePoolOptions::new()
// 		.max_connections(5)
// 		.connect_with(
// 			sqlx::sqlite::SqliteConnectOptions::new()
// 				.filename("bot.db")
// 				.create_if_missing(true),
// 		)
// 		.await
// 		.expect("Couldn't connect to database");
	
// 	database
// }

async fn get_client() -> Result<Client, Box<dyn Error>> {
	let mon_client_uri = dotenv::var("MONGODB_URI").expect("No mongodb uri");
	let options = ClientOptions::parse(&mon_client_uri).await?;
	let client = Client::with_options(options)?;
	
	Ok(client)
}