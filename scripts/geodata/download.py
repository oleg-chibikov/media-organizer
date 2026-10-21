"""Downloads GeoNames dump files into a local cache folder once."""

import shutil
import urllib.request
from pathlib import Path

BASE_URL = "https://download.geonames.org/export/dump/"
CACHE_DIR = Path(__file__).resolve().parent / ".cache"


def cached(name: str) -> Path:
    """Returns the cached file, downloading it first when it is missing."""
    CACHE_DIR.mkdir(exist_ok=True)
    target = CACHE_DIR / name
    if target.exists():
        return target
    partial = target.with_suffix(target.suffix + ".part")
    print(f"Downloading {name}")
    with urllib.request.urlopen(BASE_URL + name) as response, open(partial, "wb") as out:
        shutil.copyfileobj(response, out, length=1 << 20)
    partial.rename(target)
    return target
