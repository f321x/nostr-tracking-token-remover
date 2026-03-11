# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nostr bot (Python 3.14+) that monitors Kind 1 notes and NIP-04 encrypted DMs across multiple relays, detects tracking tokens in URLs, and replies with cleaned URLs. Uses the ClearURLs/Unalix ruleset approach for URL sanitization.

## Commands

```bash
# Run the bot (requires .env file, see .env.example)
python run_bot.py

# Run all tests
python -m unittest discover -s tests

# Run a single test file
python -m unittest tests.test_link_sanitizer

# Install dependencies
pip install .

# Docker build and run
docker build -t nostr-tracking-token-remover .
docker run --env-file .env nostr-tracking-token-remover
```

System dependency for cryptography: `libsecp256k1-dev`. Set `ELECTRUM_ECC_DONT_COMPILE=1` to use system lib instead of compiling.

## Architecture

**Entry point**: `run_bot.py` — loads env, sets up uvloop (fallback: asyncio), creates `TrackingTokenRemover` as an async context manager, waits for SIGINT/SIGTERM.

**Two-layer bot design**:
- `nostr.py::Bot` — base class handling relay connections via `electrum_aionostr.Manager`, SSL (certifi), event subscription (`subscribe_to_filter`), broadcasting, NIP-65 relay list, and Kind 0 profile publishing. Manages lifecycle through `asyncio.TaskGroup`. Crashes in the main task trigger `SIGTERM` to bring down the process.
- `bot.py::TrackingTokenRemover(Bot)` — spawns 4 concurrent tasks in the taskgroup: sanitize Kind 1 events, sanitize NIP-04 DMs, broadcast responses (rate-limited at 5s intervals via a 10k-capacity queue), and periodic status announcements.

**URL cleaning pipeline**: `link_sanitizer.py` extracts URLs via regex → `unalix_lite/url_cleaner.py` applies ClearURLs rulesets (JSON files in `rulesets/`) to strip tracking params and unwrap redirects → `difflib` computes what was removed.

**`unalix_lite/`** is a self-contained vendored module (adapted from the Unalix project). Key files: `coreutils.py` loads/compiles ruleset patterns, `url_cleaner.py` applies them, `types.py` provides URL parsing, `utils.py` handles query string filtering.

**`legacy_src/`** contains the original Rust implementation — not used, kept for reference.

## Key Nostr Protocol Details

- Kind 0: Profile metadata
- Kind 1: Public text notes (bot monitors and replies to these)
- Kind 4: NIP-04 encrypted DMs (bot decrypts, cleans, re-encrypts reply)
- Kind 10002: NIP-65 relay list advertisement
- Reply threading follows NIP-10 (root/reply `e` tags with markers)
- Events get expiration tags: 2 years for public replies, 90 days for DMs

## Environment Variables

All required vars are documented in `.env.example`. Key ones: `NOSTR_NSEC` (bot identity), `NOSTR_RELAYS` (comma-separated relay URLs), `ANNOUNCEMENT_TAG` (npub to mention in status notes), `STATUS_EVENT_INTERVAL_SEC`.
