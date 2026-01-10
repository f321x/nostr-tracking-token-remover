import re
from typing import Optional, Tuple
from .unalix_lite import clear_url


# Regex to find URLs in text
URL_REGEX = re.compile(r'(https?://\S+)')


def sanitize_urls_in_any_text(text: str) -> Optional[Tuple[str, str]]:
    """
    Finds URLs in the text, sanitizes them, and returns the cleaned text and the removed parts.
    Returns None if no tracking tokens were found.
    """
    urls = URL_REGEX.findall(text)
    if not urls:
        return None

    cleaned_urls = []
    removed_parts = []
    has_changes = False

    for url in urls:
        cleaned_url = sanitize_url(url)
        if cleaned_url != url:
            has_changes = True
            # Calculate removed part (simple diff for now, can be improved)
            # We assume the cleaned url is a prefix or substring of the original, or just different.
            # For query params, we can try to find what was removed.

            # A simple way to show what was removed is to show the original vs cleaned,
            # or just list the removed query parameters if we could parse them.
            # But here we will just return the original url as "removed part" context or similar.

            if '?' in url and '?' in cleaned_url:
                # Both have query params
                pass
            elif '?' in url:
                # Original had query params, cleaned doesn't
                removed_parts.append(url[len(cleaned_url):]) # This is a naive diff
            else:
                # Something else changed
                removed_parts.append(f"{url} -> {cleaned_url}")

            cleaned_urls.append(cleaned_url)
        else:
            cleaned_urls.append(url)

    if not has_changes:
        return None

    # Reconstruct the text with cleaned URLs
    # This is a bit tricky if there are multiple identical URLs.
    # We can use regex substitution with a callback function.

    def replace_callback(match):
        url = match.group(0)
        return sanitize_url(url)

    new_text = URL_REGEX.sub(replace_callback, text)

    # For the "removed parts" string, we want to show what was removed.
    # The rust code joins them with newlines.

    # Let's refine the removed_parts logic.
    # We need to iterate again or do it in one pass.

    removed_parts_list = []

    for url in urls:
        cleaned = sanitize_url(url)
        if cleaned != url:
            # Try to isolate what was removed
            if url.startswith(cleaned):
                removed = url[len(cleaned):]
                if removed.startswith('?'):
                    removed = removed[1:]
                elif removed.startswith('&'):
                    removed = removed[1:]
                removed_parts_list.append(removed)
            else:
                # Fallback
                removed_parts_list.append(f"Removed from {url}")

    return new_text, "\n".join(removed_parts_list)


def sanitize_url(url: str) -> str:
    try:
        return clear_url(url)
    except Exception:
        return url
