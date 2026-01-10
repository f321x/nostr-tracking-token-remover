use anyhow::Context;
use log::{debug, error, info};
use nostr_sdk::prelude::*;
use nostr_sdk::TagKind::SingleLetter;
use std::{
	collections::HashSet,
	sync::{Arc, RwLock},
};
use tokio::sync::broadcast::error::RecvError;

pub struct Bot {
	client: Client,
	keys: Keys,
	pow_enabled: bool,
	pow_difficulty: u8,
	filters: Vec<Filter>,
	filter_counter: RwLock<u64>,
	announcement_tag_npub: PublicKey,
}

// (ai)beautified message
fn format_reply_text(cleaned_url: String, diff: String) -> String {
	format!(
        "🤖 Tracking strings detected and removed!\n\n🔗 Clean URL(s):\n{}\n\n❌ Removed parts:\n{}",
        cleaned_url,
        diff
    )
}

fn sanitize_and_join_urls(note: &str) -> Option<(String, String)> {
	let sanitized = untrack::clean_urls_and_get_removed_part(note)?;
	let sanitized_urls = sanitized
		.iter()
		.map(|tuple| tuple.0.clone())
		.collect::<Vec<String>>()
		.join("\n\n");
	let removed_parts = sanitized
		.iter()
		.map(|tuple| tuple.1.clone())
		.collect::<Vec<String>>()
		.join("\n");

	Some((sanitized_urls, removed_parts))
}

impl Bot {
	pub async fn new(
		nostr_private_key: &String,
		announcement_tag_npub: &String,
		pow_enabled: bool,
		pow_difficulty: u8,
		relays: Vec<String>,
		outbox_relays: Vec<String>,
	) -> anyhow::Result<Arc<Self>> {
		let keys = Keys::parse(nostr_private_key)?;
		info!("The bot public key is: {}", keys.public_key().to_bech32()?);

		info!("Announcement tag: {}", announcement_tag_npub);
		let announcement_tag_npub = PublicKey::from_bech32(announcement_tag_npub)?;

		let client = Client::new(&keys);

		for relay in &relays {
			client
				.add_relay(relay)
				.await
				.context(format!("Error adding relay: {}", relay))?;
		}
		client.connect().await;

		let outbox_url_vec: Vec<(Url, Option<RelayMetadata>)> = outbox_relays
			.iter()
			.map(|relay| (Url::parse(relay).unwrap(), None))
			.collect();
		client.set_relay_list(outbox_url_vec).await?;

		let note_filter = Filter::new().kind(Kind::TextNote).since(Timestamp::now());
		let dm_filter = Filter::new()
			.pubkey(keys.public_key())
			.kind(Kind::EncryptedDirectMessage)
			.since(Timestamp::now());
		Ok(Arc::new(Bot {
			client,
			keys,
			pow_enabled,
			pow_difficulty,
			filters: vec![note_filter, dm_filter],
			filter_counter: RwLock::new(0),
			announcement_tag_npub,
		}))
	}

	pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
		let self_clone = self.clone();
		tokio::spawn(async move { self_clone.filter_counter_announcement_loop().await });

		let _subscription_id = self.client.subscribe(self.filters.clone(), None).await;
		let mut notifications = self.client.notifications();
		let mut replied_events: HashSet<[u8; 32]> = HashSet::new();
		let mut reconnect_counter = 0;

		loop {
			match notifications.recv().await {
				Ok(notification) => {
					if let RelayPoolNotification::Event { event, .. } = notification {
						if replied_events.contains(&event.id().to_bytes()) {
							continue;
						}
						let content = match event.kind() {
							Kind::TextNote => event.content().to_string(),
							Kind::EncryptedDirectMessage => {
								if let Ok(decrypted) = nip04::decrypt(
									self.keys.secret_key()?,
									event.author_ref(),
									event.content(),
								) {
									replied_events.insert(event.id().to_bytes());
									if let Err(e) = self
										.reply_dm_nip04(decrypted, event.author_ref(), &event.id)
										.await
									{
										error!("Error replying to DM: {}", e);
									}
									continue;
								} else {
									error!("Error decrypting DM");
									continue;
								}
							}
							_ => continue,
						};
						if let Some((link_without_tracker, diff)) = sanitize_and_join_urls(&content)
						{
							debug!("Detected tracking token: {}", &link_without_tracker);
							if let Err(e) = self.reply(link_without_tracker, diff, &event).await {
								error!("Error replying to event: {}", e);
							}

							replied_events.insert(event.id().to_bytes());
							if replied_events.len() > 1500000 {
								// 32 bytes * 1500000 = ~50 MB = forever
								replied_events.clear();
							}
						}
					}
				}
				Err(RecvError::Lagged(n)) => {
					error!("Lagged notifications: {n}");
					continue;
				}
				Err(e) => {
					error!("Error receiving notifications: {}", e);
					reconnect_counter += 1;
					self.client.unsubscribe_all().await;
					tokio::time::sleep(std::time::Duration::from_secs(5)).await;
					self.client.disconnect().await?;
					tokio::time::sleep(std::time::Duration::from_secs(30)).await;
					if reconnect_counter > 10 {
						return Err(anyhow::anyhow!("Too many reconnects"));
					}
					self.client.connect().await;
					let new_filter =
						vec![Filter::new().kind(Kind::TextNote).since(Timestamp::now())];
					let _subscription_id = self.client.subscribe(new_filter, None).await;
				}
			}
		}
	}

	#[allow(deprecated)]
	async fn reply_dm_nip04(
		&self,
		dm_content: String,
		author_pub: &PublicKey,
		reply_event_id: &EventId,
	) -> anyhow::Result<()> {
		let reply_text: String;

		if let Some((link_without_tracker, diff)) = sanitize_and_join_urls(&dm_content) {
			reply_text = format_reply_text(link_without_tracker, diff);
		} else {
			reply_text = "🤖 No tracking strings detected.".to_string();
		};
		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
		let event = match EventBuilder::encrypted_direct_msg(
			&self.keys,
			author_pub.clone(),
			reply_text,
			Some(*reply_event_id),
		) {
			Ok(event) => event,
			Err(e) => {
				return Err(anyhow::anyhow!("Error creating reply event: {}", e));
			}
		};
		let event = event
			.to_event(&self.keys)
			.context("Error signing reply event.")?;
		if let Err(e) = self.client.send_event(event).await {
			return Err(anyhow::anyhow!("Error sending reply event: {}", e));
		}
		Ok(())
	}

	async fn reply(
		&self,
		cleaned_url: String,
		diffs: String,
		event_to_reply: &Event,
	) -> anyhow::Result<()> {
		let reply_text = format_reply_text(cleaned_url, diffs);
		let reply_event = if !self.pow_enabled {
			match EventBuilder::text_note_reply(reply_text, event_to_reply, None, None)
				.to_event(&self.keys)
			{
				Ok(event) => event,
				Err(e) => {
					return Err(anyhow::anyhow!("Error creating reply event: {}", e));
				}
			}
		} else {
			let keys_clone = self.keys.clone();
			let event_to_reply = event_to_reply.clone();
			let pow_difficulty = self.pow_difficulty;
			match tokio::task::spawn_blocking(move || {
				EventBuilder::text_note_reply(reply_text, &event_to_reply, None, None)
					.to_pow_event(&keys_clone, pow_difficulty)
					.context("Error creating reply event.")
			})
			.await?
			{
				Ok(event) => event,
				Err(e) => {
					return Err(anyhow::anyhow!("Error creating reply event: {}", e));
				}
			}
		};
		if let Err(e) = self.client.send_event(reply_event).await {
			return Err(anyhow::anyhow!("Error sending reply event: {}", e));
		}
		*self.filter_counter.write().unwrap() += 1;
		Ok(())
	}

	async fn filter_counter_announcement_loop(&self) -> anyhow::Result<()> {
		loop {
			info!("Next announcement in 72 hours");
			tokio::time::sleep(std::time::Duration::from_secs(259200)).await;
			let counter = *self.filter_counter.read().unwrap();
			*self.filter_counter.write().unwrap() = 0;

			let announcement_message = format!(
				"This bot has replied to {} events with tracking tokens in the last 3 days.\nZap this bot to incentivize development.\nFind the code on GitHub: https://github.com/f321x/nostr-tracking-token-remover \n@{}",
				counter,
				self.announcement_tag_npub.to_bech32()?
			);

			let custom_tag = Tag::custom(
				SingleLetter(SingleLetterTag::lowercase(Alphabet::P)),
				vec![
					self.announcement_tag_npub.to_hex(),
					String::new(),
					"mention".to_string(),
				],
			);

			let keys = self.keys.clone();
			let announcement_event = tokio::task::spawn_blocking(move || {
				EventBuilder::text_note(announcement_message, [custom_tag])
					.to_pow_event(&keys, 24)
					.context("Error signing announcement message.")
			})
			.await??;

			if let Err(e) = self.client.send_event(announcement_event).await {
				error!("Error sending announcement event: {}", e);
			}
		}
	}
}
