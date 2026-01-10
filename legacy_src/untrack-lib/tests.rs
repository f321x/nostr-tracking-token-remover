use super::*;
use crate::parsing_params::*;

#[test]
fn test_remove_youtube_tracking_tokens() {
    let parser = Parser::new();
    // Test case 1: URL with tracking tokens
    let url_with_tokens: Url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&utm_source=newsletter&utm_medium=email").unwrap();
    let expected_clean_url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
    assert_eq!(
        parse_youtube_url(&parser, &url_with_tokens),
        Some(expected_clean_url.to_string())
    );

    // Test case 2: URL without tracking tokens
    let url_without_tokens = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
    assert_eq!(parse_youtube_url(&parser, &url_without_tokens), None);

    // Test case 3: URL with mixed parameters
    let url_mixed = Url::parse(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&t=10s&utm_source=newsletter",
    )
    .unwrap();
    let expected_mixed_clean = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
    assert_eq!(
        parse_youtube_url(&parser, &url_mixed),
        Some(expected_mixed_clean.to_string())
    );

    // Test case 4: Non-YouTube URL
    let non_youtube_url =
        Url::parse("https://www.example.com?param1=value1&utm_source=test").unwrap();
    assert_eq!(parse_youtube_url(&parser, &non_youtube_url), None);

    // Test case 5: Empty query param only
    let empty_tracking_token =
        Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ?si=").unwrap();
    assert_eq!(parse_youtube_url(&parser, &empty_tracking_token), None);

    // Test case 6: Empty query param and tracking param
    let mixed_params_url =
        Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=ABCDEFG&utm_source=").unwrap();
    let expected = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    assert_eq!(
        parse_youtube_url(&parser, &mixed_params_url),
        Some(expected.to_string())
    );
}

#[test]
fn test_twitter_url_with_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://twitter.com/user/status/123?utm_source=test&s=1").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://twitter.com/user/status/123".to_string())
    );
}

#[test]
fn test_twitter_url_without_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://twitter.com/user/status/123").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(result, None);
}

#[test]
fn test_x_com_url_with_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://x.com/user/status/123?utm_source=test&twclid=123").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(result, Some("https://x.com/user/status/123".to_string()));
}

#[test]
fn test_t_co_url_with_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://t.co/abcdef?utm_campaign=test").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(result, Some("https://t.co/abcdef".to_string()));
}

#[test]
fn test_non_twitter_url() {
    let parser = Parser::new();
    let url = Url::parse("https://example.com/page?utm_source=test").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(result, None);
}

#[test]
fn test_twitter_url_with_mixed_params() {
    let parser = Parser::new();
    let url =
        Url::parse("https://twitter.com/user/status/123?utm_source=test&valid_param=true").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://twitter.com/user/status/123?valid_param=true".to_string())
    );
}

#[test]
fn test_www_x_com_url() {
    let parser = Parser::new();
    let url = Url::parse("https://www.x.com/user/status/123?s=1&t=2").unwrap();
    let result = parse_twitter_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://www.x.com/user/status/123".to_string())
    );
}

#[test]
fn test_parse_instagram_url_valid_host_with_tracking() {
    let parser = Parser::new();
    let url = Url::parse(
        "https://www.instagram.com/p/ABC123/?utm_source=ig_web_copy_link&igshid=1234567890",
    )
    .unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://www.instagram.com/p/ABC123/".to_string())
    );
}

#[test]
fn test_parse_instagram_url_valid_host_without_tracking() {
    let parser = Parser::new();
    let url = Url::parse("https://instagram.com/user/post/123").unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(result, None);
}

#[test]
fn test_parse_instagram_url_invalid_host() {
    let parser = Parser::new();
    let url = Url::parse("https://facebook.com/instagram/post/123").unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(result, None);
}

#[test]
fn test_parse_instagram_url_with_multiple_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://www.instagram.com/reel/ABC123/?utm_source=ig_web_copy_link&igshid=1234567890&utm_medium=copy_link&_ga=GA1.2.1234567890.1234567890").unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://www.instagram.com/reel/ABC123/".to_string())
    );
}

#[test]
fn test_parse_instagram_url_with_non_tracking_params() {
    let parser = Parser::new();
    let url = Url::parse("https://www.instagram.com/p/ABC123/?hl=en&user=johndoe").unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(result, None);
}

#[test]
fn test_parse_instagram_url_with_mixed_params() {
    let parser = Parser::new();
    let url = Url::parse("https://www.instagram.com/p/ABC123/?hl=en&utm_source=ig_web&user=johndoe&igshid=1234567890").unwrap();
    let result = parse_instagram_url(&parser, &url);
    assert_eq!(
        result,
        Some("https://www.instagram.com/p/ABC123/?hl=en&user=johndoe".to_string())
    );
}

#[test]
fn test_remove_substack_tracking_tokens() {
    let parser = Parser::new();

    // Test case 1: URL with tracking tokens
    let url_with_tokens = Url::parse("https://example.substack.com/p/article-title?utm_source=newsletter&utm_medium=email&utm_campaign=promotion").unwrap();
    let expected_clean_url = Url::parse("https://example.substack.com/p/article-title").unwrap();
    assert_eq!(
        parse_substack_url(&parser, &url_with_tokens),
        Some(expected_clean_url.to_string())
    );

    // Test case 2: URL without tracking tokens
    let url_without_tokens = Url::parse("https://example.substack.com/p/article-title").unwrap();
    assert_eq!(parse_substack_url(&parser, &url_without_tokens), None);

    // Test case 3: URL with mixed parameters
    let url_mixed = Url::parse(
        "https://www.substack.com/profile/12345-author-name?utm_source=substack&r=abcde&foo=bar",
    )
    .unwrap();
    let expected_mixed_clean = "https://www.substack.com/profile/12345-author-name?foo=bar";
    assert_eq!(
        parse_substack_url(&parser, &url_mixed),
        Some(expected_mixed_clean.to_string())
    );

    // Test case 4: Non-Substack URL
    let non_substack_url =
        Url::parse("https://www.example.com?param1=value1&utm_source=test").unwrap();
    assert_eq!(parse_substack_url(&parser, &non_substack_url), None);

    // Test case 5: Substack URL with subscriber parameter
    let url_with_subscriber =
        Url::parse("https://example.substack.com/p/article-title?s=r").unwrap();
    let expected_subscriber_clean = "https://example.substack.com/p/article-title";
    assert_eq!(
        parse_substack_url(&parser, &url_with_subscriber),
        Some(expected_subscriber_clean.to_string())
    );

    // Test case 6: Substack main domain URL
    let main_domain_url = Url::parse("https://substack.com/inbox?utm_source=substack").unwrap();
    let expected_main_domain_clean = "https://substack.com/inbox";
    assert_eq!(
        parse_substack_url(&parser, &main_domain_url),
        Some(expected_main_domain_clean.to_string())
    );
}

#[test]
fn test_no_urls() {
    let mut input = String::from("This is a text without any URLs.");
    let result = replace_urls_in_place(&mut input);
    assert_eq!(result, None);
    assert_eq!(input, "This is a text without any URLs.");
}

#[test]
fn test_twitter_url() {
    let mut input = String::from(
        "Check out this tweet: https://twitter.com/user/status/123456?utm_source=test&s=12345",
    );
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "Check out this tweet: https://twitter.com/user/status/123456"
    );
}

#[test]
fn test_youtube_url() {
    let mut input = String::from("Watch this video: https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&utm_source=newsletter");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "Watch this video: https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    );
}

#[test]
fn test_substack_url() {
    let mut input = String::from("Read this article: https://example.substack.com/p/article-title?utm_source=substack&utm_medium=email");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "Read this article: https://example.substack.com/p/article-title"
    );
}

#[test]
fn test_spotify_url() {
    let mut input = String::from("Listen to this song: https://open.spotify.com/track/1234567890?si=abcdefghijklmnop&utm_source=copy-link");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "Listen to this song: https://open.spotify.com/track/1234567890"
    );
}

#[test]
fn test_instagram_url() {
    let mut input = String::from("Check out this post: https://www.instagram.com/p/ABC123/?utm_source=ig_web_copy_link&igshid=1234567890");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "Check out this post: https://www.instagram.com/p/ABC123/"
    );
}

#[test]
fn test_multiple_urls() {
    let mut input = String::from("Multiple URLs: https://twitter.com/user/status/123?s=12 and https://www.youtube.com/watch?v=abc&feature=share");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(input, "Multiple URLs: https://twitter.com/user/status/123 and https://www.youtube.com/watch?v=abc");
}

#[test]
fn test_url_with_no_tracking_params() {
    let mut input = String::from("Clean URL: https://example.com/page");
    let result = replace_urls_in_place(&mut input);
    assert_eq!(result, None);
    assert_eq!(input, "Clean URL: https://example.com/page");
}

#[test]
fn test_url_with_non_tracking_params() {
    let mut input = String::from("URL with params: https://example.com/search?q=test&page=2");
    let result = replace_urls_in_place(&mut input);
    assert_eq!(result, None);
    assert_eq!(
        input,
        "URL with params: https://example.com/search?q=test&page=2"
    );
}

#[test]
fn test_invalid_url() {
    let mut input = String::from("Invalid URL: https://not a valid url.com");
    let result = replace_urls_in_place(&mut input);
    assert_eq!(result, None);
    assert_eq!(input, "Invalid URL: https://not a valid url.com");
}

#[test]
fn test_url_with_fragment() {
    let mut input = String::from(
        "URL with fragment: https://twitter.com/user/status/123?utm_source=test#section1",
    );
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(
        input,
        "URL with fragment: https://twitter.com/user/status/123#section1"
    );
}

#[test]
fn test_url_encoding() {
    let mut input = String::from("Encoded URL: https://www.youtube.com/watch?v=abc&utm_source=newsletter&utm_campaign=summer%20sale");
    let result = replace_urls_in_place(&mut input);
    assert!(result.is_some());
    assert_eq!(input, "Encoded URL: https://www.youtube.com/watch?v=abc");
}

#[test]
fn test_no_urls2() {
    let input = String::from("This is a text without any URLs.");
    assert_eq!(clean_urls_from_any_text(&input), None);
}

#[test]
fn test_clean_url() {
    let input = String::from("Check out https://www.example.com");
    assert_eq!(clean_urls_from_any_text(&input), None);
}

#[test]
fn test_twitter_url2() {
    let input =
        String::from("Twitter link: https://twitter.com/user/status/123?utm_source=test&s=1234");
    let expected = vec!["https://twitter.com/user/status/123".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_youtube_url2() {
    let input = String::from("YouTube video: https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&utm_source=share");
    let expected = vec!["https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_substack_url2() {
    let input = String::from(
        "Substack article: https://example.substack.com/p/article?utm_source=newsletter&r=123",
    );
    let expected = vec!["https://example.substack.com/p/article".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_spotify_url2() {
    let input = String::from(
        "Spotify track: https://open.spotify.com/track/123?si=abc&utm_source=copy-link",
    );
    let expected = vec!["https://open.spotify.com/track/123".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_instagram_url2() {
    let input = String::from("Instagram post: https://www.instagram.com/p/ABC123/?utm_source=ig_web_copy_link&igshid=123");
    let expected = vec!["https://www.instagram.com/p/ABC123/".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_multiple_urls2() {
    let input = String::from(
        "Multiple URLs: https://twitter.com/user/status/123?utm_source=test \
                              https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be \
                              https://example.substack.com/p/article?utm_source=newsletter",
    );
    let expected = vec![
        "https://twitter.com/user/status/123".to_string(),
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        "https://example.substack.com/p/article".to_string(),
    ];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_mixed_clean_and_tracking_urls() {
    let input = String::from(
        "Mixed URLs: https://www.example.com \
                              https://twitter.com/user/status/123?utm_source=test \
                              https://www.cleanurl.com",
    );
    let expected = vec!["https://twitter.com/user/status/123".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_url_with_multiple_tracking_params() {
    let input = String::from("Complex URL: https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=youtu.be&utm_source=share&utm_medium=social&utm_campaign=spring2023");
    let expected = vec!["https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_url_with_non_tracking_params2() {
    let input = String::from("URL with params: https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=30s");
    assert_eq!(clean_urls_from_any_text(&input), None);
}

#[test]
fn test_malformed_url() {
    let input = String::from("Malformed URL: https://twitter com/user/status/123?utm_source=test");
    assert_eq!(clean_urls_from_any_text(&input), None);
}

#[test]
fn test_url_encoding2() {
    let input = String::from(
        "Encoded URL: https://twitter.com/search?q=rust%20programming&src=typed_query&f=live",
    );
    let expected = vec!["https://twitter.com/search?q=rust%20programming&f=live".to_string()];
    assert_eq!(clean_urls_from_any_text(&input), Some(expected));
}

#[test]
fn test_clean_urls_and_get_removed_part() {
    let input = String::from("Mixed URLs: https://www.example.com https://twitter.com/user/status/123?utm_source=test https://www.cleanurl.com");
    let expected = vec![(
        "https://twitter.com/user/status/123".to_string(),
        "?utm_source=test".to_string(),
    )];
    assert_eq!(clean_urls_and_get_removed_part(&input), Some(expected));
}
