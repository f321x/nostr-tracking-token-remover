import unittest

from nostr_tracking_token_remover.url_cleaner import clean_url


class TestCleanUrlAllowlistGate(unittest.TestCase):
    """The cleaner only acts on allowlisted big-tech / shopping domains."""

    def test_non_allowlisted_domain_is_ignored_even_with_utm(self):
        # deezer is not on the allowlist -> bot stays silent
        result = clean_url("https://deezer.com/track/123?utm_source=newsletter")
        self.assertIsNone(result)

    def test_random_blog_ref_param_is_preserved(self):
        # the classic false positive: ?ref= on a random site is NOT tracking we act on
        result = clean_url("https://someblog.example/post?ref=hackernews")
        self.assertIsNone(result)

    def test_clean_allowlisted_url_returns_none(self):
        result = clean_url("https://open.spotify.com/track/abc123")
        self.assertIsNone(result)

    def test_malformed_url_does_not_raise(self):
        # a token the URL regex might grab but urlsplit cannot parse
        self.assertIsNone(clean_url("https://[oops/path?s=1"))


class TestCleanUrlUniversalTrackers(unittest.TestCase):
    """Universal campaign / click identifiers are stripped on any allowlisted host."""

    def test_utm_stripped_on_allowlisted_host(self):
        cleaned, removed = clean_url("https://open.spotify.com/track/abc123?utm_source=newsletter")
        self.assertEqual(cleaned, "https://open.spotify.com/track/abc123")
        self.assertIn("utm_source=newsletter", removed)

    def test_fbclid_stripped_on_allowlisted_host(self):
        cleaned, removed = clean_url("https://www.facebook.com/some/post?fbclid=AbCdEf")
        self.assertEqual(cleaned, "https://www.facebook.com/some/post")
        self.assertIn("fbclid=AbCdEf", removed)


class TestCleanUrlSpotify(unittest.TestCase):
    def test_si_share_token_stripped(self):
        cleaned, removed = clean_url("https://open.spotify.com/track/abc123?si=deadbeef")
        self.assertEqual(cleaned, "https://open.spotify.com/track/abc123")
        self.assertIn("si=deadbeef", removed)


class TestCleanUrlYouTube(unittest.TestCase):
    def test_si_share_token_stripped(self):
        cleaned, removed = clean_url("https://youtu.be/dQw4w9WgXcQ?si=abcdef")
        self.assertEqual(cleaned, "https://youtu.be/dQw4w9WgXcQ")
        self.assertIn("si=abcdef", removed)

    def test_timestamp_is_preserved(self):
        # ?t= on youtube is a playback timestamp, NOT tracking -> stay silent
        result = clean_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120")
        self.assertIsNone(result)

    def test_strips_si_but_keeps_video_id_and_timestamp(self):
        cleaned, removed = clean_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120&si=abcdef"
        )
        self.assertEqual(cleaned, "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120")
        self.assertIn("si=abcdef", removed)


class TestCleanUrlTwitterX(unittest.TestCase):
    def test_x_share_tokens_stripped(self):
        # on x.com `t` is a SHARE-TRACKING token (not a timestamp) -> stripped
        cleaned, removed = clean_url("https://x.com/user/status/123?s=20&t=abcdef123")
        self.assertEqual(cleaned, "https://x.com/user/status/123")
        self.assertIn("s=20", removed)
        self.assertIn("t=abcdef123", removed)

    def test_twitter_com_also_covered(self):
        cleaned, removed = clean_url("https://twitter.com/user/status/123?s=20")
        self.assertEqual(cleaned, "https://twitter.com/user/status/123")
        self.assertIn("s=20", removed)


class TestCleanUrlReddit(unittest.TestCase):
    def test_top_time_filter_is_preserved(self):
        # reddit `t=week` is a functional time filter, NOT tracking -> stay silent
        result = clean_url("https://www.reddit.com/r/python/top/?t=week")
        self.assertIsNone(result)

    def test_share_id_stripped(self):
        cleaned, removed = clean_url(
            "https://www.reddit.com/r/python/comments/abc/title/?share_id=xyz"
        )
        self.assertEqual(cleaned, "https://www.reddit.com/r/python/comments/abc/title/")
        self.assertIn("share_id=xyz", removed)


class TestCleanUrlAmazon(unittest.TestCase):
    def test_affiliate_tag_stripped(self):
        cleaned, removed = clean_url("https://www.amazon.com/dp/B08CH7RHDP?tag=someaffiliate-20")
        self.assertEqual(cleaned, "https://www.amazon.com/dp/B08CH7RHDP")
        self.assertIn("tag=someaffiliate-20", removed)

    def test_ref_path_segment_stripped(self):
        cleaned, removed = clean_url("https://www.amazon.com/gp/B08CH7RHDP/ref=as_li_ss_tl")
        self.assertEqual(cleaned, "https://www.amazon.com/gp/B08CH7RHDP")
        self.assertIn("ref=as_li_ss_tl", removed)

    def test_search_query_is_preserved(self):
        # k (search term) and qid are functional, not tracking -> stay silent
        result = clean_url("https://www.amazon.com/s?k=mechanical+keyboard&qid=1700000000")
        self.assertIsNone(result)

    def test_works_on_regional_tld(self):
        cleaned, removed = clean_url("https://www.amazon.co.uk/dp/B08CH7RHDP?tag=aff-21")
        self.assertEqual(cleaned, "https://www.amazon.co.uk/dp/B08CH7RHDP")
        self.assertIn("tag=aff-21", removed)


class TestCleanUrlTwitch(unittest.TestCase):
    def test_uses_correct_twitch_tv_host(self):
        cleaned, removed = clean_url("https://www.twitch.tv/somestreamer?tt_medium=mobile")
        self.assertEqual(cleaned, "https://www.twitch.tv/somestreamer")
        self.assertIn("tt_medium=mobile", removed)


class TestCleanUrlYouTubePreserves(unittest.TestCase):
    def test_player_params_pp_preserved(self):
        # `pp` is opaque player params, not user tracking -> stay silent
        result = clean_url("https://www.youtube.com/watch?v=abc&pp=QAFIAQ%3D%3D")
        self.assertIsNone(result)


class TestCleanUrlGoogle(unittest.TestCase):
    def test_search_state_params_are_preserved(self):
        # ved/ei/sa are search-session state, not cross-site tracking -> silent.
        # This avoids the bot piping up on every shared Google search link.
        result = clean_url("https://www.google.com/search?q=python&ved=2ahUKEwi&ei=abc123")
        self.assertIsNone(result)

    def test_utm_still_stripped(self):
        cleaned, removed = clean_url("https://www.google.com/search?q=python&utm_source=news")
        self.assertEqual(cleaned, "https://www.google.com/search?q=python")
        self.assertIn("utm_source=news", removed)


class TestCleanUrlNetflix(unittest.TestCase):
    def test_track_params_stripped(self):
        cleaned, removed = clean_url("https://www.netflix.com/title/80100172?trackId=200257859&tctx=0")
        self.assertEqual(cleaned, "https://www.netflix.com/title/80100172")
        self.assertIn("trackId=200257859", removed)
        self.assertIn("tctx=0", removed)


class TestCleanUrlTikTok(unittest.TestCase):
    def test_share_tracking_stripped(self):
        cleaned, removed = clean_url(
            "https://www.tiktok.com/@scout/video/6718335390845095173?is_from_webapp=1&sender_device=pc"
        )
        self.assertEqual(cleaned, "https://www.tiktok.com/@scout/video/6718335390845095173")
        self.assertIn("is_from_webapp=1", removed)
        self.assertIn("sender_device=pc", removed)

    def test_lang_is_preserved(self):
        result = clean_url("https://www.tiktok.com/@scout/video/6718335390845095173?lang=en")
        self.assertIsNone(result)


class TestCleanUrlLinkedIn(unittest.TestCase):
    def test_trk_stripped(self):
        cleaned, removed = clean_url(
            "https://www.linkedin.com/feed/update/urn:li:activity:123?trk=public_post"
        )
        self.assertEqual(cleaned, "https://www.linkedin.com/feed/update/urn:li:activity:123")
        self.assertIn("trk=public_post", removed)


class TestCleanUrlApple(unittest.TestCase):
    def test_affiliate_tokens_stripped(self):
        cleaned, removed = clean_url(
            "https://music.apple.com/us/album/foo/1234567890?at=1001lABC&itscg=30200&itsct=cool"
        )
        self.assertEqual(cleaned, "https://music.apple.com/us/album/foo/1234567890")
        self.assertIn("at=1001lABC", removed)
        self.assertIn("itscg=30200", removed)

    def test_track_index_param_preserved(self):
        # `i` selects which track within an album -> functional, stay silent
        result = clean_url("https://music.apple.com/us/album/foo/1234567890?i=987654321")
        self.assertIsNone(result)


class TestCleanUrlEbay(unittest.TestCase):
    def test_tracking_stripped(self):
        cleaned, removed = clean_url(
            "https://www.ebay.com/itm/123456?_trksid=p2349624&_trkparms=abc&campid=5338"
        )
        self.assertEqual(cleaned, "https://www.ebay.com/itm/123456")
        self.assertIn("_trksid=p2349624", removed)
        self.assertIn("campid=5338", removed)


class TestCleanUrlAliExpress(unittest.TestCase):
    def test_tracking_stripped(self):
        cleaned, removed = clean_url(
            "https://www.aliexpress.com/item/1005006.html?algo_pvid=abc&aff_trace_key=xyz"
        )
        self.assertEqual(cleaned, "https://www.aliexpress.com/item/1005006.html")
        self.assertIn("algo_pvid=abc", removed)
        self.assertIn("aff_trace_key=xyz", removed)


class TestCleanUrlEtsy(unittest.TestCase):
    def test_click_tracking_stripped(self):
        cleaned, removed = clean_url(
            "https://www.etsy.com/listing/123456/cool-thing?click_key=abc&click_sum=def"
        )
        self.assertEqual(cleaned, "https://www.etsy.com/listing/123456/cool-thing")
        self.assertIn("click_key=abc", removed)
        self.assertIn("click_sum=def", removed)


class TestCleanUrlWalmart(unittest.TestCase):
    def test_attribution_stripped(self):
        cleaned, removed = clean_url(
            "https://www.walmart.com/ip/123456?athbdg=L1600&wmlspartner=abc"
        )
        self.assertEqual(cleaned, "https://www.walmart.com/ip/123456")
        self.assertIn("athbdg=L1600", removed)
        self.assertIn("wmlspartner=abc", removed)


class TestCleanUrlBestBuy(unittest.TestCase):
    def test_affiliate_tracking_stripped(self):
        cleaned, removed = clean_url(
            "https://www.bestbuy.com/site/foo/123.p?irclickid=abc&acampID=123"
        )
        self.assertEqual(cleaned, "https://www.bestbuy.com/site/foo/123.p")
        self.assertIn("irclickid=abc", removed)
        self.assertIn("acampID=123", removed)


if __name__ == "__main__":
    unittest.main()
