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

	fn parse_twitter_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let mut url = match parsed_url.host_str() {
			Some("www.twitter.com")
			| Some("twitter.com")
			| Some("t.co")
			| Some("x.com")
			| Some("www.x.com") => parsed_url.clone(),
			_ => return Ok(None),
		};

		// List of tracking parameters to remove
		let tracking_params = [
			"utm_source",
			"utm_medium",
			"utm_campaign",
			"utm_term",
			"utm_content",
			"s",
			"t",
			"src",
			"ref_src",
			"ref_url",
			"twclid",
		];

		// Get the query pairs and filter out tracking parameters
		let original_pairs: Vec<(String, String)> = url
			.query_pairs()
			.map(|(k, v)| (k.into_owned(), v.into_owned()))
			.collect();

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

	fn parse_youtube_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		// dbg!("Parsing url: {}", parsed_url.host_str().unwrap());
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

		let mut cleaned_links: Vec<String> = Vec::new();
		for link in links {
			let url = match Url::parse(link.as_str()) {
				Ok(url) => url,
				Err(_) => continue,
			};

			if let Ok(Some(youtube_link)) = self.parse_youtube_url(&url) {
				cleaned_links.push(youtube_link);
			}
			if let Ok(Some(twitter_link)) = self.parse_twitter_url(&url) {
				cleaned_links.push(twitter_link);
			}
		}
		if !cleaned_links.is_empty() {
			Ok(Some(cleaned_links.join("\n")))
		} else {
			Ok(None)
		}
	}
}

// some ai generated tests
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

	#[test]
	fn test_twitter_url_with_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://twitter.com/user/status/123?utm_source=test&s=1").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://twitter.com/user/status/123".to_string())
		);
	}

	#[test]
	fn test_twitter_url_without_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://twitter.com/user/status/123").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_x_com_url_with_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://x.com/user/status/123?utm_source=test&twclid=123").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(result, Some("https://x.com/user/status/123".to_string()));
	}

	#[test]
	fn test_t_co_url_with_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://t.co/abcdef?utm_campaign=test").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(result, Some("https://t.co/abcdef".to_string()));
	}

	#[test]
	fn test_non_twitter_url() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://example.com/page?utm_source=test").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_twitter_url_with_mixed_params() {
		let parser = Parser::new().unwrap();
		let url =
			Url::parse("https://twitter.com/user/status/123?utm_source=test&valid_param=true")
				.unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://twitter.com/user/status/123?valid_param=true".to_string())
		);
	}

	#[test]
	fn test_www_x_com_url() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://www.x.com/user/status/123?s=1&t=2").unwrap();
		let result = parser.parse_twitter_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://www.x.com/user/status/123".to_string())
		);
	}
}
