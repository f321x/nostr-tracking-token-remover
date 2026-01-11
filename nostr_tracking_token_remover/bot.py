import asyncio
import json
import time
from typing import Sequence, List, Optional

from .nostr import Bot
from .link_sanitizer import sanitize_urls_in_any_text

from electrum_aionostr.event import Event as NostrEvent


class TrackingTokenRemover(Bot):

    BROADCAST_DELAY_SEC = 5

    def __init__(
        self, *,
        relays: Sequence[str],
        nostr_nsec: str,
        nostr_profile: dict,
        status_event_interval_sec: int,
        announcement_tag: str,
    ):
        Bot.__init__(self, relays=relays, nostr_nsec=nostr_nsec)
        self._profile_info = nostr_profile
        self._status_event_interval_sec = status_event_interval_sec
        self._announcement_tag = announcement_tag
        self._response_queue = asyncio.Queue(maxsize=10_000)  # type: asyncio.Queue[NostrEvent]
        self._events_checked_count = 0
        self._events_cleaned_count = 0

    async def __aenter__(self):
        await Bot.__aenter__(self)
        assert self.taskgroup is not None
        self.taskgroup.create_task(self._sanitize_kind1_events())
        self.taskgroup.create_task(self._sanitize_nip04_dms())
        self.taskgroup.create_task(self._broadcast_status_event())
        self.taskgroup.create_task(self._broadcast_responses())
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await Bot.__aexit__(self, exc_type, exc_val, exc_tb)

    async def _sanitize_kind1_events(self):
        """
        Subscribes to all Nostr Kind 1 root events, parses them and replies with sanitized links.
        """
        query = {
            "kinds": [1],
            "limit": 0,
            "since": int(time.time()),
        }
        async for kind1_event in self.subscribe_to_filter(query):
            self._events_checked_count += 1

            result = sanitize_urls_in_any_text(kind1_event.content)
            if not result:
                continue

            cleaned_urls, removed_parts = result
            self.logger.info(f"Detected tracking token in event {kind1_event.id}")

            reply_text = self._format_reply_text(cleaned_urls, removed_parts)

            reply_tags = self._compose_reply_tags(kind1_event)

            reply_event = NostrEvent(
                kind=1,
                content=reply_text,
                tags=reply_tags,
                pubkey=self.pubkey,
            ).add_expiration_tag(
                expiration_ts=int(time.time()) + 63072000,  # 2 years
            ).sign(self._private_key.hex())

            await self._broadcast_response(reply_event)
            self._events_cleaned_count += 1

    @staticmethod
    def _compose_reply_tags(event: NostrEvent) -> List[List[str]]:
        reply_tags = []
        root_id = None
        root_relay = ""
        root_pubkey = None

        e_tags = [t for t in event.tags if len(t) > 0 and t[0] == 'e']

        for tag in e_tags:
            if len(tag) >= 4 and tag[3] == 'root':
                root_id = tag[1]
                root_relay = tag[2] if len(tag) > 2 else ""
                root_pubkey = tag[4] if len(tag) > 4 else None
                break

        if root_id is None and e_tags:
            if len(e_tags[0]) > 1:
                root_id = e_tags[0][1]
                root_relay = e_tags[0][2] if len(e_tags[0]) > 2 else ""

        if root_id:
            root_tag = ["e", root_id, root_relay, "root"]
            if root_pubkey:
                root_tag.append(root_pubkey)
            reply_tags.append(root_tag)
            reply_tags.append(["e", event.id, "", "reply", event.pubkey])
        else:
            reply_tags.append(["e", event.id, "", "root", event.pubkey])

        p_tags = [t for t in event.tags if len(t) > 1 and t[0] == 'p']
        reply_tags.extend(p_tags)

        p_pubkeys = set(t[1] for t in p_tags if len(t) > 1)

        if event.pubkey not in p_pubkeys:
            reply_tags.append(["p", event.pubkey])
            p_pubkeys.add(event.pubkey)

        if root_pubkey and root_pubkey not in p_pubkeys:
            reply_tags.append(["p", root_pubkey])

        return reply_tags

    async def _sanitize_nip04_dms(self):
        """
        Subscribes to Nostr NIP04 encrypted direct messages, parses them and replies with sanitized links.
        This allows users to use the bot privately to sanitize links.
        """
        query = {
            "kinds": [4],
            "limit": 0,
            "#p": [self.pubkey],
            "since": int(time.time()),
        }
        async for nip04_dm in self.subscribe_to_filter(query):
            self._events_checked_count += 1

            try:
                decrypted_content = self._private_key.decrypt_message(
                    encoded_message=nip04_dm.content,
                    public_key_hex=nip04_dm.pubkey,
                )
            except Exception:
                self.logger.debug(f"Failed to decrypt DM {nip04_dm.id}")
                continue

            result = sanitize_urls_in_any_text(decrypted_content)
            if result:
                cleaned_urls, removed_parts = result
                reply_text = self._format_reply_text(cleaned_urls, removed_parts)
            else:
                reply_text = "🤖 No tracking strings detected."

            # Encrypt reply
            encrypted_reply = self._private_key.encrypt_message(
                message=reply_text,
                public_key_hex=nip04_dm.pubkey,
            )

            reply_event = NostrEvent(
                kind=4,
                content=encrypted_reply,
                created_at=int(time.time()) + 2,  # add some time so it shows below the request in the chat history
                tags=[
                    ["p", nip04_dm.pubkey],
                    ["e", nip04_dm.id]
                ],
                pubkey=self.pubkey,
            ).add_expiration_tag(
                expiration_ts=int(time.time()) + 7_776_000 # 90 days
            ).sign(self._private_key.hex())
            await self._broadcast_response(reply_event)

    async def _broadcast_response(self, event: NostrEvent):
        """
        Puts the response event on the queue to be broadcast. If the queue is full we drop the
        oldest response.
        """
        if self._response_queue.full():
            event = self._response_queue.get_nowait()
            self.logger.warn(f"dropping response due to full queue: {event.id}")
        await self._response_queue.put(event)

    async def _broadcast_responses(self):
        """
        To prevent getting our IP rate limited by relays we only broadcast one response every
        self.BROADCAST_DELAY_SEC.
        """
        while True:
            response_event = await self._response_queue.get()
            await self.broadcast_nostr_event(response_event)
            await asyncio.sleep(self.BROADCAST_DELAY_SEC)

    async def _broadcast_status_event(self):
        """
        Broadcasts a summary kind 1 event every self._status_event_interval_sec.
        """
        while True:
            await asyncio.sleep(self._status_event_interval_sec)

            count_cleaned, count_checked = self._events_cleaned_count, self._events_checked_count
            self._events_cleaned_count, self._events_checked_count = 0, 0 # Reset counter

            period_days = self._status_event_interval_sec // 86400
            announcement_message = (
                f"This bot has checked {count_checked} events and found {count_cleaned} events with tracking tokens in the last {period_days} days.\n\n"
                f"Find the code on GitHub: https://github.com/f321x/nostr-tracking-token-remover"
            )

            if self._announcement_tag:
                announcement_message += f"\n@{self._announcement_tag}"

            announcement_event = NostrEvent(
                kind=1,
                content=announcement_message,
                tags=[],
                pubkey=self.pubkey,
            )
            announcement_event = announcement_event.sign(self._private_key.hex())

            await self.broadcast_nostr_event(announcement_event)

    async def get_kind0_profile_event(self) -> NostrEvent:
        profile_event = NostrEvent(
            kind=0,
            content=json.dumps(self._profile_info),
            tags=[],
            pubkey=self.pubkey,
        )
        return profile_event

    @staticmethod
    def _format_reply_text(cleaned_urls: str, diff: str) -> str:
        return (
            f"🤖 Tracking strings detected and removed!\n\n"
            f"🔗 Clean URL(s):\n{cleaned_urls}\n\n"
            f"❌ Removed parts:\n{diff}"
        )


def profile_from_env() -> dict:
    from os import environ as env
    return {
        'name': env['PROFILE_NAME'],
        'display_name': env['PROFILE_NAME'],
        'about': env['PROFILE_BIO'],
        'picture': env['PROFILE_PICTURE'],
        'website': env['PROFILE_WEBSITE'],
        'lud16': env['PROFILE_LNADDRESS'],
        'nip05': env['PROFILE_NIP05'],
    }
