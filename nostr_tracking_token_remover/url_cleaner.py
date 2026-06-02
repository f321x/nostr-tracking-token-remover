"""Precise, allowlist-based URL tracking-token remover.

Design goals (see docs / project history):

* Only act on a curated allowlist of major "big tech" and shopping domains.
  URLs on any other host are left completely untouched (the bot stays silent).
* Strip ONLY parameters that identify or attribute the user/click: campaign
  tags, click IDs, affiliate/referral tags, share-source and analytics tokens.
  Never strip functional params such as timestamps, search terms, pagination,
  language/region, or content identifiers.

Matching is by exact parameter *name* (plus a small set of explicit prefixes
like ``utm_``) against the parsed query string. There is no fuzzy/regex
matching of values, so a parameter is removed only when we are confident it is
a tracker. Because we know exactly which params we removed, callers get an
exact "was anything tracking removed?" signal instead of a length heuristic.
"""

import re
from dataclasses import dataclass
from urllib.parse import urlsplit, urlunsplit, unquote


@dataclass(frozen=True)
class _Provider:
    """A single allowlisted domain and the tracking it carries."""

    name: str
    host: re.Pattern[str]
    params: frozenset[str] = frozenset()
    prefixes: tuple[str, ...] = ()
    path_rules: tuple[re.Pattern[str], ...] = ()


def _host(*alternatives: str) -> re.Pattern[str]:
    """Compile a host matcher that matches each alternative as the registrable
    domain or any subdomain of it (anchored to the end of the host)."""
    inner = "|".join(alternatives)
    return re.compile(rf"(?:^|\.)(?:{inner})$")


# Cross-site campaign / click identifiers. Unambiguous tracking regardless of
# host, applied to every allowlisted provider.
_UNIVERSAL_PARAMS: frozenset[str] = frozenset({
    "fbclid", "gclid", "gclsrc", "dclid", "gbraid", "wbraid",
    "yclid", "msclkid", "twclid", "mc_eid", "mc_cid", "mkt_tok",
    "_openstat", "igshid", "vero_id", "vero_conv",
    "oly_anon_id", "oly_enc_id", "rb_clickid", "wickedid",
})
_UNIVERSAL_PREFIXES: tuple[str, ...] = ("utm_", "mtm_", "pk_", "matomo_", "hsa_")


# Per-domain rules. Param names are matched exactly (after percent-decoding);
# `prefixes` match name families; `path_rules` strip tracking embedded in the
# path. Only params confidently identified as tracking/attribution are listed —
# functional params (timestamps, search terms, content/variant IDs, language,
# pagination) are deliberately absent so they are preserved.
_PROVIDERS: tuple[_Provider, ...] = (
    _Provider(
        name="amazon",
        host=_host(r"amazon\.[a-z]{2,}(?:\.[a-z]{2,})?"),
        # Affiliate / ad / referral attribution. Search & variant params
        # (qid, keywords, k, sprefix, th, dib, psc, node, i, crid) are kept.
        params=frozenset({
            "tag", "ascsubtag", "linkCode", "linkId", "creativeASIN",
            "aaxitk", "hsa_cr_id", "rnid", "refRID", "adId", "spIA",
            "ms3_c", "ref", "ref_",
        }),
        prefixes=("pd_rd_", "pf_rd_"),
        path_rules=(re.compile(r"/ref=[^/?]*"),),
    ),
    _Provider(
        name="youtube",
        host=_host(r"youtube\.com", r"youtu\.be", r"youtube-nocookie\.com"),
        # NB: `t`/`start` (timestamps), `v`/`list`/`index` and `pp` (player
        # params) are intentionally absent -> preserved.
        params=frozenset({"si", "feature"}),
    ),
    _Provider(
        name="spotify",
        host=_host(r"spotify\.com"),
        params=frozenset({"si"}),
    ),
    _Provider(
        name="google",
        host=_host(r"google\.[a-z]{2,}(?:\.[a-z]{2,})?"),
        # Only the universal click IDs (gclid, utm_*, ...) apply. Search-session
        # state (q, ved, ei, sa, ...) is functional and preserved, so the bot
        # does not react to ordinary shared Google search links.
    ),
    _Provider(
        name="twitter/x",
        host=_host(r"x\.com", r"twitter\.com"),
        # `t` is a per-share tracking token (not a timestamp). `s` (e.g. s=20)
        # is the share source/surface indicator, not a per-user tracker, so it
        # is intentionally absent here -> preserved.
        params=frozenset({"t", "src", "ref_src", "refsrc", "ref_url", "cn"}),
    ),
    _Provider(
        name="facebook",
        host=_host(r"facebook\.com", r"fb\.com", r"fb\.watch"),
        params=frozenset({
            "mibextid", "_rdr", "rdr", "rdid", "comment_tracking", "ls_ref",
            "referral_code", "referral_story_type", "__tn__",
        }),
        prefixes=("__cft__", "__xts__", "ref_"),
    ),
    _Provider(
        name="instagram",
        host=_host(r"instagram\.com"),
        params=frozenset({"igshid", "igsh"}),
    ),
    _Provider(
        name="reddit",
        host=_host(r"reddit\.com", r"redd\.it"),
        # `t` (top time filter), `sort`, `context` are functional -> preserved.
        # (Keys are percent-decoded before matching, so "$" covers "%24".)
        params=frozenset({
            "share_id", "correlation_id", "ref_campaign", "ref_source", "rdt",
            "_branch_match_id", "$deep_link", "$3p", "$original_url",
        }),
    ),
    _Provider(
        name="tiktok",
        host=_host(r"tiktok\.com"),
        # Share-source tracking; the video is identified by the path, not these.
        params=frozenset({
            "is_from_webapp", "sender_device", "_r", "_t", "tt_from",
            "share_app_id", "share_link_id", "share_app_name",
        }),
    ),
    _Provider(
        name="linkedin",
        host=_host(r"linkedin\.com"),
        params=frozenset({"trk", "trackingId", "lipi", "li_fat_id"}),
    ),
    _Provider(
        name="apple",
        host=_host(r"apple\.com", r"apple\.co"),
        # Affiliate / campaign tokens. `i` (track), `mt`, `l` are functional.
        params=frozenset({"at", "ct", "itscg", "itsct", "mttnsubad"}),
    ),
    _Provider(
        name="netflix",
        host=_host(r"netflix\.com"),
        params=frozenset({"trackId", "tctx"}),
    ),
    _Provider(
        name="twitch",
        host=_host(r"twitch\.tv"),
        params=frozenset({"tt_medium", "tt_content"}),
    ),
    _Provider(
        name="ebay",
        host=_host(r"ebay\.[a-z]{2,}(?:\.[a-z]{2,})?"),
        params=frozenset({
            "_trkparms", "_trksid", "mkcid", "mkrid", "campid", "customid",
            "mkevt", "toolid",
        }),
    ),
    _Provider(
        name="aliexpress",
        host=_host(r"aliexpress\.[a-z]{2,}(?:\.[a-z]{2,})?"),
        params=frozenset({
            "ws_ab_test", "btsid", "algo_expid", "algo_pvid", "gps-id", "cv",
            "af", "mall_affr", "sk", "terminal_id", "ali_refid", "ali_trackid",
        }),
        prefixes=("scm", "aff_"),
    ),
    _Provider(
        name="etsy",
        host=_host(r"etsy\.com"),
        params=frozenset({"click_key", "click_sum", "organic_search_click", "frs"}),
    ),
    _Provider(
        name="walmart",
        host=_host(r"walmart\.com"),
        params=frozenset({"wmlspartner", "sourceid", "u1"}),
        prefixes=("ath",),
    ),
    _Provider(
        name="bestbuy",
        host=_host(r"bestbuy\.com"),
        params=frozenset({
            "irclickid", "acampID", "irgwc", "afsrc", "mpid", "affgroup",
        }),
    ),
)


def _provider_for(host: str) -> _Provider | None:
    for provider in _PROVIDERS:
        if provider.host.search(host):
            return provider
    return None


def _is_tracking(key: str, provider: _Provider) -> bool:
    name = unquote(key)
    if name in provider.params or name in _UNIVERSAL_PARAMS:
        return True
    return name.startswith(provider.prefixes + _UNIVERSAL_PREFIXES)


def clean_url(url: str) -> tuple[str, list[str]] | None:
    """Remove tracking tokens from *url* if its host is allowlisted.

    Returns ``(cleaned_url, removed)`` where ``removed`` is the list of removed
    query segments / path fragments (e.g. ``"utm_source=foo"``). Returns
    ``None`` if the host is not allowlisted or nothing tracking was removed.
    """
    try:
        parts = urlsplit(url)
        host = parts.hostname or ""
    except ValueError:
        return None  # not a parseable URL -> leave it alone

    provider = _provider_for(host)
    if provider is None:
        return None

    removed: list[str] = []

    new_path = parts.path
    for rule in provider.path_rules:
        while (match := rule.search(new_path)) is not None:
            removed.append(new_path[match.start():match.end()].lstrip("/"))
            new_path = new_path[:match.start()] + new_path[match.end():]

    new_query = parts.query
    if parts.query:
        kept = []
        for segment in parts.query.split("&"):
            if not segment:
                continue
            key = segment.split("=", 1)[0]
            if _is_tracking(key, provider):
                removed.append(segment)
            else:
                kept.append(segment)
        new_query = "&".join(kept)

    if not removed:
        return None

    cleaned = urlunsplit(
        (parts.scheme, parts.netloc, new_path, new_query, parts.fragment)
    )
    return cleaned, removed
