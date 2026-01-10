//! This crate is intended to remove tracking tokens from URLs contained in any text input.
//! The crate can either substitute the URLs in place or return a Vec of cleaned urls to a given text input.

mod parsing_core;
mod parsing_params;

#[cfg(test)]
mod tests;

use linkify::{LinkFinder, LinkKind};
use parsing_core::Parser;
use std::collections::HashSet;
use url::Url;

/// Takes any String as input, parses URLs, returns either `None` if no tracking tokens
/// were found. Otherwise returns `Some(Vec<String>)` of all sanitized URLs
pub fn clean_urls_from_any_text(input: &str) -> Option<Vec<String>> {
    let parser = Parser::new();
    let cleaned_urls = parser.parse_any_text(input);
    if cleaned_urls.is_none() {
        return None;
    }
    cleaned_urls.map(|urls| {
        urls.into_iter()
            .map(|(cleaned_url, _)| cleaned_url)
            .collect()
    })
}

/// Same as clean_urls_from_any_text, but returns Tuples of the sanitized URL and the part that was removed
pub fn clean_urls_and_get_removed_part(input: &str) -> Option<Vec<(String, String)>> {
    let parser = Parser::new();
    let cleaned_urls = parser.parse_any_text(input);
    cleaned_urls
}

/// Parses any (mutable) String and sanitizes URLs containing tracking tokens in place
pub fn replace_urls_in_place(input: &mut String) -> Option<&mut String> {
    let parser = Parser::new();
    let changes = parser.sanitize_in_place(input);
    if changes.is_some() {
        Some(input)
    } else {
        None
    }
}

/// Sanitizes the input and returns `Some<String>` if any changes were made
pub fn clone_and_sanitize_text(input: &str) -> Option<String> {
    let mut cloned_input = String::from(input);
    let parser = Parser::new();
    let changes = parser.sanitize_in_place(&mut cloned_input);
    if changes.is_some() {
        Some(cloned_input)
    } else {
        None
    }
}
