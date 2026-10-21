"""Collects the GeoNames ids that have an English Wikipedia article."""

import io
import zipfile

from download import cached

WIKIPEDIA_PREFIXES = ("https://en.wikipedia.org/", "http://en.wikipedia.org/")


def linked_ids() -> set[int]:
    """Streams alternateNamesV2, where a `link` row holds a URL for a GeoNames id."""
    ids = set()
    archive = cached("alternateNamesV2.zip")
    with zipfile.ZipFile(archive) as zipped, zipped.open("alternateNamesV2.txt") as raw:
        for line in io.TextIOWrapper(raw, encoding="utf-8"):
            fields = line.split("\t", 4)
            if fields[2] == "link" and fields[3].startswith(WIKIPEDIA_PREFIXES):
                ids.add(int(fields[1]))
    return ids
