from typing import Optional, Tuple
import re

from .url_cleaner import clean_url


# Regex to find URLs in text.
# Matches http/https URLs, excluding trailing punctuation (.,;:!?) followed by
# whitespace or end of string.
URL_REGEX = re.compile(r'(https?://\S+?)(?=[.,;:!?]*(?:\s|$))')


def sanitize_urls_in_any_text(text: str) -> Optional[Tuple[str, str]]:
    """
    Finds URLs in the text, removes tracking tokens from allowlisted hosts, and
    returns the cleaned URLs and the removed parts. URLs on non-allowlisted
    hosts (and URLs without tracking tokens) are ignored.

    Returns None if no tracking tokens were found in any allowlisted URL.
    """
    cleaned_urls = []
    removed_parts = []

    for url in URL_REGEX.findall(text):
        result = clean_url(url)
        if result is None:
            continue
        cleaned_url, removed = result
        cleaned_urls.append(cleaned_url)
        removed_parts.extend(removed)

    if not cleaned_urls:
        return None

    return "\n".join(cleaned_urls), "\n".join(removed_parts)
