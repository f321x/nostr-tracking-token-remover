use super::*;
use crate::parsing_params::*;

pub struct Parser {
    finder: LinkFinder,
}

impl Parser {
    pub fn new() -> Self {
        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);
        Self { finder }
    }

    pub fn parse_any_text(&self, input: &str) -> Option<Vec<(String, String)>> {
        let links: Vec<_> = self.finder.links(input).collect();
        if links.is_empty() {
            return None;
        }

        let mut cleaned_links: Vec<(String, String)> = Vec::new();
        for link in links {
            let url = match Url::parse(link.as_str()) {
                Ok(url) => url,
                Err(_) => continue,
            };

            let cleaned_url: String;
            if let Some(youtube_link) = parse_youtube_url(self, &url) {
                cleaned_url = youtube_link;
            } else if let Some(twitter_link) = parse_twitter_url(self, &url) {
                cleaned_url = twitter_link;
            } else if let Some(instagram_link) = parse_instagram_url(self, &url) {
                cleaned_url = instagram_link;
            } else if let Some(spotify_link) = parse_spotify_url(self, &url) {
                cleaned_url = spotify_link;
            } else if let Some(substack_link) = parse_substack_url(self, &url) {
                cleaned_url = substack_link;
            } else {
                continue;
            }

            let diff_to_original: String = diff::chars(&cleaned_url, url.as_str())
                .into_iter()
                .filter_map(|result| match result {
                    diff::Result::Right(r) => Some(r.to_string()),
                    _ => None,
                })
                .collect::<Vec<String>>()
                .join("");

            cleaned_links.push((cleaned_url, diff_to_original));
        }
        if !cleaned_links.is_empty() {
            Some(cleaned_links)
        } else {
            None
        }
    }

    pub fn sanitize_in_place(&self, input: &mut String) -> Option<()> {
        let original = input.clone();
        let parsed_links = self.parse_any_text(&original);

        if let Some(cleaned_links) = parsed_links {
            let mut finder = LinkFinder::new();
            finder.kinds(&[LinkKind::Url]);
            let mut last_end = 0;
            let mut result = String::new();

            for link in finder.links(&original) {
                let start = link.start();
                let end = link.end();

                // Add the text before the link
                result.push_str(&original[last_end..start]);

                // Find the corresponding cleaned link
                if let Some(cleaned_link) = cleaned_links.iter().find(|&cl| {
                    let original_url = Url::parse(link.as_str()).ok();
                    let cleaned_url = Url::parse(&cl.0).ok();
                    match (original_url, cleaned_url) {
                        (Some(ou), Some(cu)) => ou.path() == cu.path() && ou.host() == cu.host(),
                        _ => false,
                    }
                }) {
                    result.push_str(&cleaned_link.0);
                } else {
                    // If no cleaned version found, keep the original link
                    result.push_str(link.as_str());
                }

                last_end = end;
            }

            // Add any remaining text after the last link
            result.push_str(&original[last_end..]);

            // Update the input string
            *input = result;

            Some(())
        } else {
            None
        }
    }

    pub fn parse_url(
        &self,
        parsed_url: &Url,
        valid_hosts: &[&str],
        tracking_params: &[&str],
    ) -> Option<String> {
        if !valid_hosts.contains(&parsed_url.host_str().unwrap_or("")) {
            return None;
        }

        let mut url = parsed_url.clone();
        let tracking_params: HashSet<_> = tracking_params.iter().cloned().collect();

        let original_pairs: Vec<(String, String)> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        // Check if there are any non-empty query parameters that are not tracking params
        if original_pairs.iter().all(|(_, v)| v.is_empty()) {
            return None;
        }

        let filtered_pairs: Vec<(String, String)> = original_pairs
            .iter()
            .filter(|(key, _)| !tracking_params.contains(key.as_str()))
            .cloned()
            .collect();

        if original_pairs.len() == filtered_pairs.len() {
            return None;
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

        Some(url.to_string())
    }
}
