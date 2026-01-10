mod bot;

#[cfg(test)]
mod bot_tests;

use anyhow::Result;
use bot::Bot;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Debug)
		.init();
	dotenv().ok();

	let relays = env::var("NOSTR_RELAYS")?
		.split(',')
		.map(|s| s.to_string())
		.collect();
	let outbox_relays = env::var("OUTBOX_RELAYS")?
		.split(',')
		.map(|s| s.to_string())
		.collect();

	let bot = Bot::new(
		&env::var("NOSTR_NSEC")?,
		&env::var("ANNOUNCEMENT_TAG")?,
		env::var("POW_MODE")?.parse()?,
		env::var("POW_DIFFICULTY")?.parse()?,
		relays,
		outbox_relays,
	)
	.await?;
	bot.run().await?;

	Ok(())
}
