import asyncio
import json
import time
from typing import Sequence

from .nostr import Bot
from .link_sanitizer import sanitize_urls_in_any_text

from electrum_aionostr.event import Event as NostrEvent


class TrackingTokenRemover(Bot):

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
        self._events_cleaned_count = 0

    async def __aenter__(self):
        await Bot.__aenter__(self)
        assert self.taskgroup is not None
        self.taskgroup.create_task(self._sanitize_kind1_events())
        self.taskgroup.create_task(self._sanitize_nip04_dms())
        self.taskgroup.create_task(self._broadcast_status_event())
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
        }
        async for kind1_event in self.subscribe_to_filter(query):
            result = sanitize_urls_in_any_text(kind1_event.content)
            if not result:
                continue

            cleaned_urls, removed_parts = result
            self.logger.debug(f"Detected tracking token in event {kind1_event.id}")

            reply_text = self._format_reply_text(cleaned_urls, removed_parts)

            reply_tags = [
                ['e', kind1_event.id],
            ]

            reply_event = NostrEvent(
                kind=1,
                content=reply_text,
                tags=reply_tags,
                pubkey=self.pubkey,
            ).add_expiration_tag(
                expiration_ts=int(time.time()) + 63072000,  # 2 years
            ).sign(self._private_key.hex())

            await self.broadcast_nostr_event(reply_event)
            self._events_cleaned_count += 1

    async def _sanitize_nip04_dms(self):
        """
        Subscribes to Nostr NIP04 encrypted direct messages, parses them and replies with sanitized links.
        This allows users to use the bot privately to sanitize links.
        """
        query = {
            "kinds": [4],
            "limit": 0,
            "tags": ["#p", self.pubkey],
        }
        async for nip04_dm in self.subscribe_to_filter(query):
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
                tags=[
                    ["p", nip04_dm.pubkey],
                    ["e", nip04_dm.id]
                ],
                pubkey=self.pubkey,
            ).add_expiration_tag(
                expiration_ts=int(time.time()) + 7_776_000 # 90 days
            ).sign(self._private_key.hex())
            await self.broadcast_nostr_event(reply_event)

    async def _broadcast_status_event(self):
        """
        Broadcasts a summary kind 1 event every self._status_event_interval_sec.
        """
        while True:
            await asyncio.sleep(self._status_event_interval_sec)

            count = self._events_cleaned_count
            self._events_cleaned_count = 0 # Reset counter

            announcement_message = (
                f"This bot has replied to {count} events with tracking tokens in the last period.\n\n"
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
