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
		"Hey, the link you shared contained tracking tokens.\n
        Here is a link without tracking tokens:\n
        {} \n
        Please consider zapping this bot to keep it alive and report bugs via DM",
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
					let _ = self.reply(&link_without_tracker, &event).await;
				}
			}
		}
		Ok(())
	}

	async fn reply(&self, cleaned_url: &str, event_to_reply: &Event) -> Result<()> {
		let reply_text = format_reply_text(cleaned_url);
		let reply_event =
			match EventBuilder::text_note_reply(reply_text, event_to_reply, None, None)
				.to_event(&self.keys)
			{
				Ok(event) => event,
				Err(e) => {
					println!("Error creating reply event: {}", e);
					return Ok(());
				}
			};
		if let Err(e) = self.client.send_event(reply_event).await {
			println!("Error publishing reply event: {}", e);
		}
		Ok(())
	}
}
