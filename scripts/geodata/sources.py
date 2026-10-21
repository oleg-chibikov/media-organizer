"""Streams GeoNames dumps and yields the rows the asset keeps."""

import io
import unicodedata
from collections.abc import Collection
import zipfile
from typing import Iterator, NamedTuple

from download import cached

# Neighbourhoods and places that no longer exist make poor folder names.
SKIPPED_CITY_CODES = {"PPLX", "PPLH", "PPLQ", "PPLW", "PPLCH"}
MIN_CITY_POPULATION = 1000


class Row(NamedTuple):
    geoname_id: int
    feature_class: str
    name: str
    latitude: float
    longitude: float
    feature_code: str
    country_code: str
    admin1_code: str
    population: int


def countries() -> dict[str, str]:
    """Maps ISO 3166 alpha-2 codes to English country names."""
    result = {}
    with open(cached("countryInfo.txt"), encoding="utf-8") as lines:
        for line in lines:
            if line.startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t")
            result[fields[0]] = _ascii(fields[4])
    return result


def _ascii(text: str) -> str:
    """Drops accents so names like Curacao stay safe in folder names."""
    decomposed = unicodedata.normalize("NFKD", text)
    return decomposed.encode("ascii", "ignore").decode("ascii")


def admin1_names() -> dict[str, str]:
    """Maps `CC.code` keys to ASCII region names."""
    result = {}
    with open(cached("admin1CodesASCII.txt"), encoding="utf-8") as lines:
        for line in lines:
            key, _name, ascii_name, _id = line.rstrip("\n").split("\t")
            result[key] = ascii_name
    return result


def cities() -> Iterator[Row]:
    for row in _rows("cities1000.zip", "cities1000.txt"):
        if row.feature_code in SKIPPED_CITY_CODES:
            continue
        if row.population >= MIN_CITY_POPULATION:
            yield row


def features(wanted: set[tuple[str, str]]) -> Iterator[Row]:
    """Streams the large allCountries dump without loading it into memory."""
    for row in _rows("allCountries.zip", "allCountries.txt", {c for c, _ in wanted}):
        if (row.feature_class, row.feature_code) in wanted:
            yield row


def _rows(archive: str, member: str, classes: Collection[str] = ("P",)) -> Iterator[Row]:
    with zipfile.ZipFile(cached(archive)) as zipped, zipped.open(member) as raw:
        for line in io.TextIOWrapper(raw, encoding="utf-8"):
            fields = line.split("\t")
            name = fields[2].strip()
            if fields[6] not in classes or not name.isascii() or not name:
                continue
            yield Row(
                geoname_id=int(fields[0]),
                feature_class=fields[6],
                name=name,
                latitude=float(fields[4]),
                longitude=float(fields[5]),
                feature_code=fields[7],
                country_code=fields[8],
                admin1_code=fields[10],
                population=int(fields[14] or 0),
            )
