mod bot;

use anyhow::Result;
use bot::Bot;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.init();
	dotenv().ok();

	let bot = Bot::new(&env::var("NOSTR_NSEC")?, &env::var("ANNOUNCEMENT_TAG")?).await?;
	bot.run().await?;

	Ok(())
}
