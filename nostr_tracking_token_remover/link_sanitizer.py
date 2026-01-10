import re
import difflib
from typing import Optional, Tuple

from .unalix_lite import clear_url


# Regex to find URLs in text
# Matches http/https URLs, excluding trailing punctuation (.,;:!?) followed by whitespace or end of string.
URL_REGEX = re.compile(r'(https?://\S+?)(?=[.,;:!?]*(?:\s|$))')


def sanitize_urls_in_any_text(text: str) -> Optional[Tuple[str, str]]:
    """
    Finds URLs in the text, sanitizes them, and returns the cleaned text and the removed parts.
    Returns None if no tracking tokens were found.
    """
    urls = URL_REGEX.findall(text)
    if not urls:
        return None

    cleaned_urls = []
    removed_parts_list = []
    has_changes = False

    for url in urls:
        cleaned_url = sanitize_url(url)
        if do_urls_differ(url, cleaned_url):
            has_changes = True
            cleaned_urls.append(cleaned_url)
            if removed := get_removed_part(url, cleaned_url):
                removed_parts_list.append(removed)

    if not has_changes:
        return None

    return "\n".join(cleaned_urls), "\n".join(removed_parts_list)


def sanitize_url(url: str) -> str:
    try:
        return clear_url(url)
    except Exception:
        return url


def do_urls_differ(original_url: str, cleaned_url: str) -> bool:
    if original_url == cleaned_url:
        return False

    if abs(len(original_url) - len(cleaned_url)) <= 3:
        return False  # allow 3 char diff, might just be some normalization going on

    return True


def get_removed_part(original_url: str, cleaned_url: str) -> Optional[str]:
    """
    Compares original and cleaned URL to determine what was removed.
    Returns the diff of the both urls or None if they are similar.
    """
    assert original_url != cleaned_url

    if original_url.startswith(cleaned_url):
        return original_url[len(cleaned_url):]

    # Use SequenceMatcher to find differences
    matcher = difflib.SequenceMatcher(None, original_url, cleaned_url)

    # Collect removed parts (parts present in original but not in cleaned)
    removed_parts = []
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == 'delete':
            # Part only in original URL
            removed_parts.append(original_url[i1:i2])
        elif tag == 'replace':
            # Different parts - show what was in original
            removed_parts.append(original_url[i1:i2])

    if removed_parts:
        return ''.join(removed_parts)

    # URLs are completely different (e.g., redirect unwrapping)
    return None

