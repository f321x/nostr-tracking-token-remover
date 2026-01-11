"""Vibecoded unittests"""

import unittest
from nostr_tracking_token_remover.bot import TrackingTokenRemover
from electrum_aionostr.event import Event as NostrEvent

# Valid 32-byte hex strings for testing
PUBKEY_ROOT = "0000000000000000000000000000000000000000000000000000000000000001"
PUBKEY_REPLY = "0000000000000000000000000000000000000000000000000000000000000002"
PUBKEY_OTHER = "0000000000000000000000000000000000000000000000000000000000000003"
ID_ROOT = "0000000000000000000000000000000000000000000000000000000000000010"
ID_PARENT = "0000000000000000000000000000000000000000000000000000000000000011"

class TestReplyTags(unittest.TestCase):

    def test_reply_to_root_event(self):
        # Case 1: Replying to a root event (no e tags)
        original_event = NostrEvent(
            kind=1,
            content="Root event content",
            tags=[],
            pubkey=PUBKEY_ROOT,
        )

        self.assertIsNotNone(original_event.id)

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        # Expect:
        # - e tag pointing to root event as "root"
        # - p tag for the author of the root event
        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_reply_to_reply_event_with_root_marker(self):
        # Case 2: Replying to a reply event that has a marked root
        original_event = NostrEvent(
            kind=1,
            content="Reply content",
            tags=[
                ["e", ID_ROOT, "wss://relay.com", "root", PUBKEY_ROOT],
                ["e", ID_PARENT, "", "reply", PUBKEY_OTHER],
                ["p", PUBKEY_ROOT],
                ["p", PUBKEY_OTHER]
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags_start = [
            ["e", ID_ROOT, "wss://relay.com", "root", PUBKEY_ROOT],
            ["e", original_event.id, "", "reply", PUBKEY_REPLY],
            ["p", PUBKEY_ROOT],
            ["p", PUBKEY_OTHER]
        ]

        self.assertEqual(reply_tags[:4], expected_tags_start)

        p_tags = [t for t in reply_tags if t[0] == 'p']
        p_values = [t[1] for t in p_tags]
        self.assertIn(PUBKEY_REPLY, p_values)
        self.assertIn(PUBKEY_ROOT, p_values)
        self.assertIn(PUBKEY_OTHER, p_values)

    def test_reply_to_reply_event_deprecated_positional(self):
        # Case 3: Replying to a reply event using deprecated positional tags
        original_event = NostrEvent(
            kind=1,
            content="Reply content",
            tags=[
                ["e", ID_ROOT, "wss://relay.com"],
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags_start = [
            ["e", ID_ROOT, "wss://relay.com", "root"],
            ["e", original_event.id, "", "reply", PUBKEY_REPLY]
        ]

        self.assertEqual(reply_tags[:2], expected_tags_start)

        p_tags = [t for t in reply_tags if t[0] == 'p']
        p_values = [t[1] for t in p_tags]
        self.assertIn(PUBKEY_REPLY, p_values)

    def test_malformed_tags_empty_e_tag(self):
        # Case 4: Malformed tags - empty e tag
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e"],
                ["p", PUBKEY_OTHER]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_OTHER],
            ["p", PUBKEY_ROOT]
        ]

        self.assertEqual(reply_tags, expected_tags)

    def test_malformed_tags_short_e_tag(self):
        # Case 5: e tag with only id
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e", ID_ROOT],
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags_start = [
            ["e", ID_ROOT, "", "root"],
            ["e", original_event.id, "", "reply", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags[:2], expected_tags_start)

    def test_root_marker_extraction(self):
        # Case 7: Extracting root marker correctly
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e", ID_ROOT, "", "root", PUBKEY_ROOT],
                ["e", ID_PARENT, "", "reply", PUBKEY_OTHER]
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        self.assertEqual(reply_tags[0], ["e", ID_ROOT, "", "root", PUBKEY_ROOT])
        self.assertEqual(reply_tags[1], ["e", original_event.id, "", "reply", PUBKEY_REPLY])

    def test_p_tag_deduplication(self):
        # Case 8: Ensure p tags are not duplicated if author is already in p tags
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["p", PUBKEY_ROOT]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_root_pubkey_addition_to_p_tags(self):
        # Case 9: If root pubkey is found in e tag, it should be added to p tags if missing
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e", ID_ROOT, "", "root", PUBKEY_ROOT]
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        p_tags = [t for t in reply_tags if t[0] == 'p']
        p_values = [t[1] for t in p_tags]

        self.assertIn(PUBKEY_REPLY, p_values)
        self.assertIn(PUBKEY_ROOT, p_values)

    def test_malformed_p_tag_empty(self):
        # Case 10: Empty p tag
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["p"]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        # Should ignore empty p tag, treat as root event
        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_malformed_p_tag_extra_data(self):
        # Case 11: p tag with extra data
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["p", PUBKEY_OTHER, "relay", "name"]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        # Should preserve the extra data in p tag
        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_OTHER, "relay", "name"],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_malformed_multiple_e_tags_first_empty(self):
        # Case 12: First e tag is empty, second is valid
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e"],
                ["e", ID_ROOT]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        # Should fallback to treating current event as root because first e tag is invalid positional root
        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_root_marker_missing_pubkey(self):
        # Case 13: e tag has root marker but no pubkey
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e", ID_ROOT, "", "root"]
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags = [
            ["e", ID_ROOT, "", "root"],
            ["e", original_event.id, "", "reply", PUBKEY_REPLY],
            ["p", PUBKEY_REPLY]
        ]
        self.assertEqual(reply_tags, expected_tags)

    def test_root_marker_extra_data(self):
        # Case 14: e tag has root marker and extra data
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["e", ID_ROOT, "", "root", PUBKEY_ROOT, "extra_stuff"]
            ],
            pubkey=PUBKEY_REPLY,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        # Should strip extra data when constructing the new root tag
        expected_tags = [
            ["e", ID_ROOT, "", "root", PUBKEY_ROOT],
            ["e", original_event.id, "", "reply", PUBKEY_REPLY],
            ["p", PUBKEY_ROOT],
            ["p", PUBKEY_REPLY]
        ]

        # Note: order of p tags depends on implementation, but here we check existence
        self.assertEqual(reply_tags[:2], expected_tags[:2])

        p_tags = [t for t in reply_tags if t[0] == 'p']
        p_values = [t[1] for t in p_tags]
        self.assertIn(PUBKEY_ROOT, p_values)
        self.assertIn(PUBKEY_REPLY, p_values)

    def test_other_tags_ignored(self):
        # Case 15: Other tags (e.g. hashtags) are ignored
        original_event = NostrEvent(
            kind=1,
            content="Content",
            tags=[
                ["t", "hashtag"]
            ],
            pubkey=PUBKEY_ROOT,
        )

        reply_tags = TrackingTokenRemover._compose_reply_tags(original_event)

        expected_tags = [
            ["e", original_event.id, "", "root", PUBKEY_ROOT],
            ["p", PUBKEY_ROOT]
        ]
        self.assertEqual(reply_tags, expected_tags)
