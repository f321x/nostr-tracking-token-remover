#!/usr/bin/env python3

import asyncio
import signal
import logging
from os import environ as env

from nostr_tracking_token_remover import TrackingTokenRemover, profile_from_env

from dotenv import load_dotenv

_HAS_UVLOOP = False
try:
    import uvloop
    _HAS_UVLOOP = True
except ImportError:
    pass


def set_up_logger(log_level: str):
    log_level = log_level.upper()
    assert log_level in ('INFO', 'DEBUG', 'WARN', 'ERROR'), f"invalid log level: {log_level}"
    logging.basicConfig(
        level=getattr(logging, log_level),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

async def main():
    load_dotenv()
    relays: list[str] = [r.strip() for r in env['NOSTR_RELAYS'].split(',')]
    nostr_nsec: str = env['NOSTR_NSEC'].strip()

    log_level: str = env.get('LOG_LEVEL', 'INFO')
    set_up_logger(log_level)
    logger = logging.getLogger('nostr-tracking-token-bot')
    logger.info(
        f"ENV:\n{relays=}\n{log_level=}"
    )

    shutdown_event = asyncio.Event()
    loop = asyncio.get_running_loop()
    loop.add_signal_handler(signal.SIGINT, shutdown_event.set)  # type: ignore
    loop.add_signal_handler(signal.SIGTERM, shutdown_event.set)  # type: ignore
    async with TrackingTokenRemover(
        relays=relays,
        nostr_nsec=nostr_nsec,
        nostr_profile=profile_from_env(),
        status_event_interval_sec=int(env['STATUS_EVENT_INTERVAL_SEC']),
    ) as _bot:
        logger.info(f"tracking token remover running")
        await shutdown_event.wait()
    await asyncio.sleep(0.25)
    logger.info(f"tracking token remover stopped")

if __name__ == "__main__":
    if _HAS_UVLOOP:
        uvloop.run(main())
    else:
        asyncio.run(main())
