# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nostr bot (Python 3.14+) that monitors Kind 1 notes and NIP-04 encrypted DMs across multiple relays, detects tracking tokens in URLs, and replies with cleaned URLs. URL cleaning is intentionally conservative: it only acts on a curated allowlist of major "big tech" and shopping domains, and only strips parameters known to be tracking/attribution tokens (never timestamps, search terms, or other functional params).

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

**URL cleaning pipeline**: `link_sanitizer.py` extracts URLs from text via regex → `url_cleaner.py::clean_url` cleans each one. Both Kind 1 and NIP-04 DM handlers use the same `sanitize_urls_in_any_text` entry point, so there is a single cleaning path.

**`url_cleaner.py`** is the whole cleaning engine. It holds an `_PROVIDERS` allowlist (one `_Provider` per supported domain, with a host regex, a set of exact tracking param names, optional name prefixes like `pf_rd_`, and optional path rules like Amazon's `/ref=…`) plus a `_UNIVERSAL_PARAMS`/`_UNIVERSAL_PREFIXES` set (e.g. `utm_*`, `fbclid`, `gclid`) applied to every allowlisted host. `clean_url(url)` returns `(cleaned_url, removed_parts)` only when the host is allowlisted **and** a known tracker was removed, otherwise `None`. Matching is by exact param name (not fuzzy value matching), so the "did we remove tracking?" signal is exact — there is no length/diff heuristic. To support a new domain or token, edit `_PROVIDERS` and add a test in `tests/test_url_cleaner.py`.

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
