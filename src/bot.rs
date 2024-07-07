use crate::parsing::Parser;
use nostr_sdk::prelude::*;

pub struct Bot {
	client: Client,
	keys: Keys,
	filters: Vec<Filter>,
	parser: Parser,
}

fn format_reply_text(cleaned_url: &str) -> String {
	format!(
		"Hey, the link you shared contains tracking tokens.
		Here is a link without tracking tokens:
		{}
		Zap this bot to keep it alive and report bugs on Github",
		cleaned_url
	)
}

impl Bot {
	pub async fn new(nostr_private_key: &String) -> anyhow::Result<Self> {
		let keys = Keys::parse(nostr_private_key)?;
		println!("The bot public key is: {}", keys.public_key().to_bech32()?);

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
		client.connect().await;

		let note_filter = Filter::new().kind(Kind::TextNote).since(Timestamp::now());
		Ok(Bot {
			client,
			keys,
			filters: vec![note_filter],
			parser: Parser::new()?,
		})
	}

	pub async fn run(&self) -> anyhow::Result<()> {
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
}
