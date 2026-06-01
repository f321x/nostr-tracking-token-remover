import unittest

from nostr_tracking_token_remover.link_sanitizer import sanitize_urls_in_any_text


class TestLinkSanitizer(unittest.TestCase):

    def test_no_urls(self):
        self.assertIsNone(sanitize_urls_in_any_text("just some text, no links here"))

    def test_clean_allowlisted_url_returns_none(self):
        self.assertIsNone(
            sanitize_urls_in_any_text("listen: https://open.spotify.com/track/abc123")
        )

    def test_non_allowlisted_dirty_url_is_ignored(self):
        # deezer is not on the big-tech allowlist -> stay silent even with utm
        text = "Check this out: https://deezer.com/track/891177062?utm_source=deezer"
        self.assertIsNone(sanitize_urls_in_any_text(text))

    def test_allowlisted_dirty_url(self):
        text = "song: https://open.spotify.com/track/abc123?si=deadbeef"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned, removed = result
        self.assertEqual(cleaned, "https://open.spotify.com/track/abc123")
        self.assertIn("si=deadbeef", removed)

    def test_trailing_punctuation_excluded(self):
        text = "song: https://open.spotify.com/track/abc123?si=deadbeef. Nice"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned, _removed = result
        self.assertEqual(cleaned, "https://open.spotify.com/track/abc123")

    def test_multiple_allowlisted_urls(self):
        text = (
            "a https://open.spotify.com/track/abc123?si=x "
            "b https://www.amazon.com/dp/B08CH7RHDP?tag=aff-20"
        )
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned, removed = result
        self.assertEqual(
            cleaned,
            "https://open.spotify.com/track/abc123\n"
            "https://www.amazon.com/dp/B08CH7RHDP",
        )
        self.assertIn("si=x", removed)
        self.assertIn("tag=aff-20", removed)

    def test_mixed_clean_and_dirty_reports_only_dirty(self):
        text = (
            "clean https://open.spotify.com/track/clean "
            "dirty https://www.youtube.com/watch?v=abc&si=track"
        )
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned, removed = result
        self.assertEqual(cleaned, "https://www.youtube.com/watch?v=abc")
        self.assertIn("si=track", removed)

    def test_mixed_allowlisted_and_non_allowlisted_reports_only_allowlisted(self):
        text = (
            "blog https://someblog.example/post?utm_source=x "
            "yt https://youtu.be/abc?si=track"
        )
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned, removed = result
        self.assertEqual(cleaned, "https://youtu.be/abc")
        self.assertIn("si=track", removed)


if __name__ == "__main__":
    unittest.main()
