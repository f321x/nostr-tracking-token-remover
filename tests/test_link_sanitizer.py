import unittest
from nostr_tracking_token_remover.link_sanitizer import sanitize_urls_in_any_text


class TestLinkSanitizer(unittest.TestCase):

    def test_sanitize_urls_in_any_text_no_urls(self):
        text = "This is a text without urls."
        result = sanitize_urls_in_any_text(text)
        self.assertIsNone(result)

    def test_sanitize_urls_in_any_text_clean_url(self):
        text = "Check this out: https://google.com"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNone(result)

    def test_sanitize_urls_in_any_text_dirty_url(self):
        text = "Check this out: https://deezer.com/track/891177062?utm_source=deezer"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned_text, removed_parts = result
        self.assertEqual(cleaned_text, "Check this out: https://deezer.com/track/891177062")
        self.assertIn("utm_source=deezer", removed_parts)

    def test_sanitize_urls_in_any_text_dirty_url_dot_at_end(self):
        text = "Check this out: https://deezer.com/track/891177062?utm_source=deezer. Test"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned_text, removed_parts = result
        self.assertEqual(cleaned_text, "Check this out: https://deezer.com/track/891177062. Test")
        self.assertIn("utm_source=deezer", removed_parts)

    def test_sanitize_urls_in_any_text_multiple_urls(self):
        text = "Two links: https://deezer.com/track/891177062?utm_source=deezer and https://www.amazon.com/gp/B08CH7RHDP/ref=as_li_ss_tl"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned_text, removed_parts = result
        self.assertEqual(cleaned_text, "Two links: https://deezer.com/track/891177062 and https://www.amazon.com/gp/B08CH7RHDP")
        self.assertIn("utm_source=deezer", removed_parts)
        self.assertIn("ref=as_li_ss_tl", removed_parts)

    def test_sanitize_urls_in_any_text_mixed_urls(self):
        text = "One clean https://google.com and one dirty https://deezer.com/track/891177062?utm_source=deezer"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned_text, removed_parts = result
        self.assertEqual(cleaned_text, "One clean https://google.com and one dirty https://deezer.com/track/891177062")
        self.assertIn("utm_source=deezer", removed_parts)

    def test_sanitize_urls_in_any_text_redirection(self):
        text = "Redirect: https://www.google.com/url?q=https://pypi.org/project/Unalix"
        result = sanitize_urls_in_any_text(text)
        self.assertIsNotNone(result)
        cleaned_text, removed_parts = result
        self.assertEqual(cleaned_text, "Redirect: https://pypi.org/project/Unalix")
        # The removed part logic for redirection might be tricky, let's see what it produces
        # It should probably show the original url or the diff.
        # In our implementation:
        # if url.startswith(cleaned): ... else: removed_parts_list.append(f"Removed from {url}")
        # Since cleaned is not a prefix of url (it's completely different), it should fall back.
        self.assertIn("Removed from https://www.google.com/url?q=https://pypi.org/project/Unalix", removed_parts)
