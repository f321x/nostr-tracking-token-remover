use super::*;
use linkify::{LinkFinder, LinkKind};
use std::collections::HashSet;
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
			if let Ok(Some(instagram_link)) = self.parse_instagram_url(&url) {
				cleaned_links.push(instagram_link);
			}
			if let Ok(Some(spotify_link)) = self.parse_spotify_url(&url) {
				cleaned_links.push(spotify_link);
			}
			if let Ok(Some(substack_link)) = self.parse_substack_url(&url) {
				cleaned_links.push(substack_link);
			}
		}
		if !cleaned_links.is_empty() {
			Ok(Some(cleaned_links.join("\n\n")))
		} else {
			Ok(None)
		}
	}

	fn parse_twitter_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let valid_hosts = [
			"www.twitter.com",
			"twitter.com",
			"t.co",
			"x.com",
			"www.x.com",
		];
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
		self.parse_url(parsed_url, &valid_hosts, &tracking_params)
	}

	fn parse_youtube_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let valid_hosts = [
			"www.youtube.com",
			"youtube.com",
			"youtu.be",
			"yt.be",
			"m.youtube.com",
			"music.youtube.com",
		];
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
		self.parse_url(parsed_url, &valid_hosts, &tracking_params)
	}

	fn parse_substack_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let tracking_params = [
			"utm_source",
			"utm_medium",
			"utm_campaign",
			"utm_term",
			"utm_content",
			"source",
			"r", // referral parameter
			"s", // subscriber parameter
		];

		let host = parsed_url.host_str().unwrap_or("");

		if host == "www.substack.com" || host == "substack.com" || host.ends_with(".substack.com") {
			self.parse_url(parsed_url, &[host], &tracking_params)
		} else {
			Ok(None)
		}
	}

	fn parse_spotify_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let valid_hosts = [
			"open.spotify.com",
			"play.spotify.com",
			"spotify.com",
			"www.spotify.com",
			"artist.spotify.com",
			"embed.spotify.com",
		];
		let tracking_params = [
			"si", // Spotify Identifier
			"utm_source",
			"utm_medium",
			"utm_campaign",
			"utm_term",
			"utm_content",
			"feature",
			"nd", // No Delay
			"context",
			"context_id",
			"sp_cid",  // Spotify Campaign ID
			"sp_ac",   // Spotify Ad Click
			"sp_gaid", // Google Advertising ID
			"sp_aid",  // Apple Identifier for Advertisers
			"go",      // Generic Origin
			"fbclid",  // Facebook Click Identifier
			"product",
			"referral",
		];
		self.parse_url(parsed_url, &valid_hosts, &tracking_params)
	}

	fn parse_instagram_url(&self, parsed_url: &Url) -> anyhow::Result<Option<String>> {
		let valid_hosts = ["www.instagram.com", "instagram.com"];
		let tracking_params = [
			"utm_source",
			"utm_medium",
			"utm_campaign",
			"utm_term",
			"utm_content",
			"igshid",
			"fbclid",
			"_ga",
			"_gid",
		];
		self.parse_url(parsed_url, &valid_hosts, &tracking_params)
	}

	fn parse_url(
		&self,
		parsed_url: &Url,
		valid_hosts: &[&str],
		tracking_params: &[&str],
	) -> anyhow::Result<Option<String>> {
		if !valid_hosts.contains(&parsed_url.host_str().unwrap_or("")) {
			return Ok(None);
		}

		let mut url = parsed_url.clone();
		let tracking_params: HashSet<_> = tracking_params.iter().cloned().collect();

		let original_pairs: Vec<(String, String)> = url
			.query_pairs()
			.map(|(k, v)| (k.into_owned(), v.into_owned()))
			.collect();

		let filtered_pairs: Vec<(String, String)> = original_pairs
			.iter()
			.filter(|(key, _)| !tracking_params.contains(key.as_str()))
			.cloned()
			.collect();

		if original_pairs.len() == filtered_pairs.len() {
			return Ok(None);
		}

		url.set_query(None);

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

	#[test]
	fn test_parse_instagram_url_valid_host_with_tracking() {
		let parser = Parser::new().unwrap();
		let url = Url::parse(
			"https://www.instagram.com/p/ABC123/?utm_source=ig_web_copy_link&igshid=1234567890",
		)
		.unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://www.instagram.com/p/ABC123/".to_string())
		);
	}

	#[test]
	fn test_parse_instagram_url_valid_host_without_tracking() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://instagram.com/user/post/123").unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_instagram_url_invalid_host() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://facebook.com/instagram/post/123").unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_instagram_url_with_multiple_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://www.instagram.com/reel/ABC123/?utm_source=ig_web_copy_link&igshid=1234567890&utm_medium=copy_link&_ga=GA1.2.1234567890.1234567890").unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://www.instagram.com/reel/ABC123/".to_string())
		);
	}

	#[test]
	fn test_parse_instagram_url_with_non_tracking_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://www.instagram.com/p/ABC123/?hl=en&user=johndoe").unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_instagram_url_with_mixed_params() {
		let parser = Parser::new().unwrap();
		let url = Url::parse("https://www.instagram.com/p/ABC123/?hl=en&utm_source=ig_web&user=johndoe&igshid=1234567890").unwrap();
		let result = parser.parse_instagram_url(&url).unwrap();
		assert_eq!(
			result,
			Some("https://www.instagram.com/p/ABC123/?hl=en&user=johndoe".to_string())
		);
	}

	#[test]
	fn test_remove_substack_tracking_tokens() {
		let parser = Parser::new().unwrap();

		// Test case 1: URL with tracking tokens
		let url_with_tokens = Url::parse("https://example.substack.com/p/article-title?utm_source=newsletter&utm_medium=email&utm_campaign=promotion").unwrap();
		let expected_clean_url =
			Url::parse("https://example.substack.com/p/article-title").unwrap();
		assert_eq!(
			parser.parse_substack_url(&url_with_tokens).unwrap(),
			Some(expected_clean_url.to_string())
		);

		// Test case 2: URL without tracking tokens
		let url_without_tokens =
			Url::parse("https://example.substack.com/p/article-title").unwrap();
		assert_eq!(
			parser.parse_substack_url(&url_without_tokens).unwrap(),
			None
		);

		// Test case 3: URL with mixed parameters
		let url_mixed = Url::parse("https://www.substack.com/profile/12345-author-name?utm_source=substack&r=abcde&foo=bar").unwrap();
		let expected_mixed_clean = "https://www.substack.com/profile/12345-author-name?foo=bar";
		assert_eq!(
			parser.parse_substack_url(&url_mixed).unwrap(),
			Some(expected_mixed_clean.to_string())
		);

		// Test case 4: Non-Substack URL
		let non_substack_url =
			Url::parse("https://www.example.com?param1=value1&utm_source=test").unwrap();
		assert_eq!(parser.parse_substack_url(&non_substack_url).unwrap(), None);

		// Test case 5: Substack URL with subscriber parameter
		let url_with_subscriber =
			Url::parse("https://example.substack.com/p/article-title?s=r").unwrap();
		let expected_subscriber_clean = "https://example.substack.com/p/article-title";
		assert_eq!(
			parser.parse_substack_url(&url_with_subscriber).unwrap(),
			Some(expected_subscriber_clean.to_string())
		);

		// Test case 6: Substack main domain URL
		let main_domain_url = Url::parse("https://substack.com/inbox?utm_source=substack").unwrap();
		let expected_main_domain_clean = "https://substack.com/inbox";
		assert_eq!(
			parser.parse_substack_url(&main_domain_url).unwrap(),
			Some(expected_main_domain_clean.to_string())
		);
	}
}
