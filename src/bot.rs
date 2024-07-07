use crate::parsing::Parser;
use anyhow::Context;
use nostr_sdk::prelude::*;
use std::cell::{Ref, RefCell};

pub struct Bot {
	client: Client,
	keys: Keys,
	filters: Vec<Filter>,
	parser: Parser,
	filter_counter: RefCell<u64>,
	announcement_tag_npub: PublicKey,
}

fn format_reply_text(cleaned_url: &str) -> String {
	format!(
		"Hey, the link you shared contains tracking tokens.\nHere is a link without tracking tokens:\n{}\nZap this bot to keep it alive and report bugs on Github",
		cleaned_url
	)
}

impl Bot {
	pub async fn new(
		nostr_private_key: &String,
		announcement_tag_npub: &String,
	) -> anyhow::Result<Self> {
		let keys = Keys::parse(nostr_private_key)?;
		println!("The bot public key is: {}", keys.public_key().to_bech32()?);

		let announcement_tag_npub = PublicKey::from_bech32(announcement_tag_npub)?;

		let client = Client::new(&keys);

		client.add_relay("wss://relay.damus.io").await?;
		client.add_relay("wss://relay.primal.net").await?;
		client.add_relay("wss://relay.nostr.band").await?;
		client
			.add_relay("wss://ftp.halifax.rwth-aachen.de/nostr")
			.await?;
		client.add_relay("wss://nostr.mom").await?;
		client.add_relay("wss://relay.nostrplebs.com").await?;
		client.add_relay("wss://relay.mostr.pub").await?;
		client.add_relay("wss://nos.lol").await?;
		client.add_relay("wss://nostr.einundzwanzig.space").await?;
		client.add_relay("wss://relay.snort.social").await?;
		client.add_relay("wss://nostr.land").await?;
		client.add_relay("wss://nostr.oxtr.dev").await?;
		client.add_relay("wss://nostr.fmt.wiz.biz").await?;
		client.add_relay("wss://bitcoiner.social").await?;
		client.connect().await;

		let note_filter = Filter::new().kind(Kind::TextNote).since(Timestamp::now());
		Ok(Bot {
			client,
			keys,
			filters: vec![note_filter],
			parser: Parser::new()?,
			filter_counter: RefCell::new(0),
			announcement_tag_npub,
		})
	}

	pub async fn run(&self) -> anyhow::Result<()> {
		self.filter_counter_announcement_loop().await?;
		let _subscription_id = self.client.subscribe(self.filters.clone(), None).await;
		let mut notifications = self.client.notifications();

		while let Ok(notification) = notifications.recv().await {
			if let RelayPoolNotification::Event { event, .. } = notification {
				if let Some(link_without_tracker) =
					self.parser.parse_event_content(event.content())?
				{
					println!("Detected tracking token: {}", &link_without_tracker);
					if let Err(e) = self.reply(&link_without_tracker, &event).await {
						println!("Error replying to event: {}", e);
					}
				}
			}
		}
		Err(anyhow::anyhow!("Bot stopped running"))
	}

	async fn reply(&self, cleaned_url: &str, event_to_reply: &Event) -> anyhow::Result<()> {
		*self.filter_counter.borrow_mut() += 1;
		let reply_text = format_reply_text(cleaned_url);
		let reply_event =
			match EventBuilder::text_note_reply(reply_text, event_to_reply, None, None)
				.to_event(&self.keys)
			{
				Ok(event) => event,
				Err(e) => {
					return Err(anyhow::anyhow!("Error creating reply event: {}", e));
				}
			};
		if let Err(e) = self.client.send_event(reply_event).await {
			return Err(anyhow::anyhow!("Error sending reply event: {}", e));
		}
		Ok(())
	}

	async fn filter_counter_announcement_loop(&self) -> anyhow::Result<()> {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
			let counter = *self.filter_counter.borrow();
			*self.filter_counter.borrow_mut() = 0;

			let announcement_message = format!(
				"This bot has replied to {} events with tracking tokens in the last 24 hours.\nZap this bot to incentivize developement.\nFind the code on GitHub: https://github.com/f321x/nostr-tracking-token-remover",
				counter
			);
			let announcement_event = EventBuilder::text_note(
				announcement_message,
				[Tag::public_key(self.announcement_tag_npub)],
			)
			.to_event(&self.keys)
			.context("Error signing announcement message.")?;
			if let Err(e) = self.client.send_event(announcement_event).await {
				println!("Error sending announcement event: {}", e);
			}
		}
	}
}
