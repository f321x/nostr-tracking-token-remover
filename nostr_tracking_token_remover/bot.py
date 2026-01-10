from typing import Sequence

from .nostr import Bot

from electrum_aionostr.event import Event as NostrEvent


class TrackingTokenRemover(Bot):

    def __init__(
        self, *,
        relays: Sequence[str],
        nostr_nsec: str,
        nostr_profile: dict,
        status_event_interval_sec: int,
    ):
        Bot.__init__(self, relays=relays, nostr_nsec=nostr_nsec)
        self._profile_info = nostr_profile
        self._status_event_interval_sec = status_event_interval_sec

    async def __aenter__(self):
        await AIONostrDVM.__aenter__(self)
        assert self.taskgroup is not None
        self.taskgroup.create_task(self._sanitize_kind1_events())
        self.taskgroup.create_task(self._sanitize_nip04_dms())
        self.taskgroup.create_task(self._broadcast_status_event())
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await AIONostrDVM.__aexit__(self, exc_type, exc_val, exc_tb)

    async def _sanitize_kind1_events(self):
        """
        Subscribes to all Nostr Kind 1 root events, parses them and replies with sanitized links.
        """
        query = {
            "kinds": [1],
            "limit": 0,
        }
        async for kind1_event in self.subscribe_to_filter(query):
            pass

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
            pass

    async def _broadcast_status_event(self):
        """
        Broadcasts a summary kind 1 event every self._status_event_interval_sec.
        """
        pass

    async def get_kind0_profile_event(self) -> NostrEvent:
        profile_event = NostrEvent(
            kind=0,
            content=json.dumps(self._profile_info),
            tags=[],
            pubkey=self.pubkey,
        )
        return profile_event

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
