use super::*;
use crate::parsing_core::Parser;

pub fn parse_twitter_url(parser: &Parser, parsed_url: &Url) -> Option<String> {
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
    parser.parse_url(parsed_url, &valid_hosts, &tracking_params)
}

pub fn parse_youtube_url(parser: &Parser, parsed_url: &Url) -> Option<String> {
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
    parser.parse_url(parsed_url, &valid_hosts, &tracking_params)
}

pub fn parse_substack_url(parser: &Parser, parsed_url: &Url) -> Option<String> {
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
        parser.parse_url(parsed_url, &[host], &tracking_params)
    } else {
        None
    }
}

pub fn parse_spotify_url(parser: &Parser, parsed_url: &Url) -> Option<String> {
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
    parser.parse_url(parsed_url, &valid_hosts, &tracking_params)
}

pub fn parse_instagram_url(parser: &Parser, parsed_url: &Url) -> Option<String> {
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
    parser.parse_url(parsed_url, &valid_hosts, &tracking_params)
}
