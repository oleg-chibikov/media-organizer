"""Turns one Overture park into one asset line, in UTF-8.

`kind name area_km2 lat lon rings`, tab separated. Rings are Google polyline
strings at 1e-5 degrees, separated by spaces. Outer rings and holes are mixed,
since an even-odd test treats them the same.
"""

import json
import re

from anyascii import anyascii

KIND_BY_CLASS = {"national_park": "national_park", "state_park": "reserve",
                 "wilderness_area": "reserve"}


# Chinese, Japanese and Korean. Transliteration joins their words and reads
# Japanese as Chinese, so these names stay as written.
CJK = re.compile(r"[\u1100-\u11ff\u3040-\u30ff\u3400-\u9fff\uac00-\ud7af]")
QUOTES = str.maketrans("", "", "«»\"“”„")


def folder_name(name: str) -> str:
    """Latin letters for every script but CJK, as in `Ropotamo` for `Ропотамо`."""
    name = name.translate(QUOTES)
    return name if CJK.search(name) else anyascii(name)


def park_line(name: str, park_class: str, km2: float, label: str, outline: str) -> str | None:
    name = " ".join(folder_name(name).split())
    if not name:
        return None
    longitude, latitude = json.loads(label)["coordinates"]
    geometry = json.loads(outline)
    polygons = geometry["coordinates"]
    if geometry["type"] == "Polygon":
        polygons = [polygons]
    rings = " ".join(_encode(ring) for polygon in polygons for ring in polygon if len(ring) >= 4)
    if not rings:
        return None
    return (f"{KIND_BY_CLASS[park_class]}\t{name}\t{km2:.0f}"
            f"\t{latitude:.5f}\t{longitude:.5f}\t{rings}")


def _encode(ring: list[list[float]]) -> str:
    chars = []
    last = (0, 0)
    for longitude, latitude in ring:
        point = (round(latitude * 1e5), round(longitude * 1e5))
        if point == last:
            continue
        for delta in (point[0] - last[0], point[1] - last[1]):
            _encode_number(delta, chars)
        last = point
    return "".join(chars)


def _encode_number(value: int, chars: list[str]) -> None:
    value = ~(value << 1) if value < 0 else value << 1
    while value >= 0x20:
        chars.append(chr((0x20 | (value & 0x1F)) + 63))
        value >>= 5
    chars.append(chr(value + 63))
