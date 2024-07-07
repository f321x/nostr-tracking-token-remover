use super::*;
use linkify::{LinkFinder, LinkKind};
use url::Url;

pub struct Parser {
	finder: LinkFinder,
}

impl Parser {
	pub fn new() -> Result<Self> {
		let mut finder = LinkFinder::new();
		finder.kinds(&[LinkKind::Url]);
		Ok(Self { finder })
	}

	fn parse_youtube_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		dbg!("Parsing url: {}", parsed_url.host_str().unwrap());
		let mut url = match parsed_url.host_str() {
			Some("www.youtube.com") | Some("youtube.com") | Some("youtu.be") | Some("yt.be") => {
				parsed_url.clone()
			}
			_ => return Ok(None),
		};

		// List of tracking parameters to remove
		let tracking_params = [
			"utm_source",
			"utm_medium",
			"utm_campaign",
			"utm_term",
			"utm_content",
			"feature",
			"gclid",
			"fbclid",
			"si",
			"pp",
		];

		// Get the query pairs and filter out tracking parameters
		let original_pairs: Vec<(String, String)> = url
			.query_pairs()
			.map(|(k, v)| (k.into_owned(), v.into_owned()))
			.collect();

		// Get the query pairs and filter out tracking parameters
		let filtered_pairs: Vec<(String, String)> = url
			.query_pairs()
			.filter(|(key, _)| !tracking_params.contains(&key.as_ref()))
			.map(|(k, v)| (k.into_owned(), v.into_owned()))
			.collect();

		// If no tracking tokens were removed, return None
		if original_pairs.len() == filtered_pairs.len() {
			return Ok(None);
		}
		// Clear the existing query string
		url.set_query(None);

		// Add back the filtered parameters
		if !filtered_pairs.is_empty() {
			let query_string = filtered_pairs
				.into_iter()
				.map(|(k, v)| format!("{}={}", k, v))
				.collect::<Vec<String>>()
				.join("&");
			url.set_query(Some(&query_string));
		}

		Ok(Some(url.to_string()))
	}

	pub fn parse_event_content(&self, event_content: &str) -> anyhow::Result<Option<String>> {
		let links: Vec<_> = self.finder.links(event_content).collect();
		if links.is_empty() {
			return Ok(None);
		}

		for link in links {
			let url = match Url::parse(link.as_str()) {
				Ok(url) => url,
				Err(_) => continue,
			};

			if let Ok(Some(youtube_link)) = self.parse_youtube_url(&url) {
				return Ok(Some(youtube_link));
			}
		}

		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_remove_youtube_tracking_tokens() {
		let parser = Parser::new().unwrap();
		// Test case 1: URL with tracking tokens
		let url_with_tokens: Url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&utm_source=newsletter&utm_medium=email").unwrap();
		let expected_clean_url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
		assert_eq!(
			parser.parse_youtube_url(&url_with_tokens).unwrap(),
			Some(expected_clean_url.to_string())
		);

		// Test case 2: URL without tracking tokens
		let url_without_tokens = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
		assert_eq!(parser.parse_youtube_url(&url_without_tokens).unwrap(), None);

		// Test case 3: URL with mixed parameters
		let url_mixed = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&t=10s&utm_source=newsletter").unwrap();
		let expected_mixed_clean = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
		assert_eq!(
			parser.parse_youtube_url(&url_mixed).unwrap(),
			Some(expected_mixed_clean.to_string())
		);

		// Test case 4: Non-YouTube URL
		let non_youtube_url =
			Url::parse("https://www.example.com?param1=value1&utm_source=test").unwrap();
		assert_eq!(parser.parse_youtube_url(&non_youtube_url).unwrap(), None);
	}
}
