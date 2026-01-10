import asyncio
import json
import logging
import os
import ssl
import signal
from contextlib import asynccontextmanager

import certifi
from typing import Optional, AsyncGenerator, Sequence

import electrum_aionostr
from electrum_aionostr.event import Event as NostrEvent
from electrum_aionostr.key import PrivateKey


class Bot:

    CONNECTION_TIMEOUT_SEC = 60

    def __init__(
        self, *,
        relays: Sequence[str],
        nostr_nsec: str,
    ):
        self.logger = logging.getLogger('nostr-bot')
        self._private_key = PrivateKey.from_nsec(nostr_nsec)
        self.pubkey = self._private_key.public_key.hex()
        self.relays = set(normalize_websocket_urls(relays))
        self.taskgroup = None  # type: Optional[asyncio.TaskGroup]
        self._main_task = None  # type: Optional[asyncio.Task]
        self._relay_manager = None  # type: Optional[electrum_aionostr.Manager]
        self._initialized = asyncio.Event()

    async def __aenter__(self) -> 'Bot':
        self._main_task = asyncio.create_task(self.main_loop())
        self._main_task.add_done_callback(self._on_main_task_done)
        await asyncio.wait_for(self._initialized.wait(), timeout=self.CONNECTION_TIMEOUT_SEC + 5)
        assert isinstance(self._relay_manager, electrum_aionostr.Manager)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        try:
            await asyncio.wait_for(self.stop(), timeout=10)
        except asyncio.TimeoutError:
            return

    async def get_kind0_profile_event(self) -> Optional[electrum_aionostr.event.Event]:
        """To be implemented by child. If the dvm should publish a kind0 profile event this function
        should return a kind 0 event object, otherwise it should return None"""
        raise NotImplementedError("child must implement get_kind0_profile_event()")

    def _on_main_task_done(self, task: asyncio.Task) -> None:
        if task.cancelled():
            return
        exc = task.exception()
        if exc is not None:
            # rip down the whole thing if the main loop crashed
            self.logger.exception("main task crashed", exc_info=exc)
            try:
                os.kill(os.getpid(), signal.SIGTERM)
            except Exception:
                self.logger.exception("failed to send SIGTERM after crash")

    async def main_loop(self) -> None:
        async with self._start_relay_manager() as _manager:
            self.logger.debug(f"starting AIONostrDVM taskgroup")
            try:
                async with asyncio.TaskGroup() as tg:
                    self.taskgroup = tg
                    self._initialized.set()
                    tg.create_task(self._broadcast_nip65_relay_announcement_event())
                    await asyncio.sleep(10)
                    tg.create_task(self._broadcast_kind0_profile_event())
            except* ValueError as eg:
                # re-raise the first exception happening in the taskgroup
                self.logger.exception("Task group failed")
                raise eg.exceptions[0]
            finally:
                self.taskgroup = None
                self.logger.debug(f"taskgroup stopped")

    @asynccontextmanager
    async def _start_relay_manager(self) -> AsyncGenerator[electrum_aionostr.Manager, None]:
        ca_path = certifi.where()
        ssl_context = ssl.create_default_context(purpose=ssl.Purpose.SERVER_AUTH, cafile=ca_path)
        self.logger.info(f"connecting to nostr relays")
        manager_logger = logging.getLogger('bot-relay-manager')
        try:
            async with electrum_aionostr.Manager(
                relays=self.relays,
                private_key=self._private_key.hex(),
                ssl_context=ssl_context,
                connect_timeout=self.CONNECTION_TIMEOUT_SEC,
                log=manager_logger,
            ) as relay_manager:
                self._relay_manager = relay_manager
                yield relay_manager
        except Exception:
            self.logger.exception(f"relay manager crashed")
            raise
        finally:
            self._relay_manager = None
            self.logger.debug(f"relay manager closed")

    async def stop(self) -> None:
        if self._main_task:
            self._main_task.cancel()
        while self._relay_manager is not None:
            self.logger.debug(f"waiting for relay manager disconnect")
            await asyncio.sleep(2)

    async def subscribe_to_filter(self, query: dict) -> AsyncGenerator[NostrEvent, None]:
        assert self._relay_manager is not None and self._relay_manager.connected
        async for event in self._relay_manager.get_events(query, single_event=False, only_stored=False):
            yield event
            await asyncio.sleep(0.1)

    async def broadcast_nostr_event(self, event: NostrEvent):
        try:
            event_id = await self._relay_manager.add_event(event)
            self.logger.debug(f"broadcasted event: {event_id}")
        except Exception:
            self.logger.error(f"failed to broadcast event: {event.id}")

    async def _broadcast_nip65_relay_announcement_event(self):
        """This allows other clients to know which relays we are using (NIP-65)"""
        tags = [['r', relay_url] for relay_url in self.relays]
        nip65_event = NostrEvent(
            kind=10002,
            content='',
            tags=tags,
            pubkey=self.pubkey,
        )
        nip65_event = nip65_event.sign(self._private_key.hex())
        await self.broadcast_nostr_event(nip65_event)

    async def _broadcast_kind0_profile_event(self):
        """Broadcast the kind0 profile event once"""
        profile_event = await self.get_kind0_profile_event()
        while not profile_event:
            await asyncio.sleep(30)
            profile_event = await self.get_kind0_profile_event()
        assert profile_event.kind == 0
        profile_event = profile_event.sign(self._private_key.hex())
        try:
            await self._relay_manager.add_event(profile_event)
            self.logger.debug(f"broadcast kind 0 profile event")
        except Exception:
            self.logger.error(f"failed to broadcast kind 0 profile event")


def normalize_websocket_urls(urls: Sequence[str]) -> list[str]:
    normalized = []
    for url in urls:
        url = url.strip().lower()
        if not url.startswith(('ws://', 'wss://')):
            url = 'wss://' + url
        if url.endswith('/'):
            url = url[:-1]
        normalized.append(url)
    return normalized
