mod bot;
mod parsing;

use anyhow::Result;
use bot::Bot;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
	dotenv().ok();
	let bot = Bot::new(&env::var("NOSTR_NSEC")?).await?;
	bot.run().await?;

	Ok(())
}
