pub mod meme;
pub mod youtube;
pub mod poketcg;

// Database queries
async fn get_database_connection() -> sqlx::Pool<sqlx::Sqlite> {
	let database = sqlx::sqlite::SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(
			sqlx::sqlite::SqliteConnectOptions::new()
				.filename("bot.db")
				.create_if_missing(true),
		)
		.await
		.expect("Couldn't connect to database");
	
	database
}