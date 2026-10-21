"""Builds src-tauri/assets/geodata.tsv.gz from GeoNames dumps (CC BY 4.0).

Run from the repo root: python3 scripts/geodata/build_geodata.py
Downloads land in scripts/geodata/.cache and stay out of git.
"""

import gzip
import zlib
from collections import Counter, defaultdict
from pathlib import Path

import kinds
import sources
import wikipedia

ASSET = Path(__file__).resolve().parents[2] / "src-tauri" / "assets" / "geodata.tsv.gz"


def collect_places(countries: dict[str, str]) -> list[tuple[str, sources.Row]]:
    places = [("city", row) for row in sources.cities()]
    linked = wikipedia.linked_ids()
    for row in sources.features(set(kinds.KIND_BY_FEATURE)):
        kind = kinds.kind_of(row)
        if kind and kinds.is_notable(kind, row, linked):
            places.append((kind, row))
    # Rows outside any country, such as Antarctic features, have no folder to go to.
    places = [place for place in places if place[1].country_code in countries]
    places.sort(key=lambda place: (place[0], place[1].country_code, place[1].admin1_code,
                                   place[1].name, place[1].geoname_id))
    return places


def asset_lines(countries: dict[str, str], places: list[tuple[str, sources.Row]]) -> list[str]:
    """Writes countries, then admin1 names, then places that point at them by index."""
    admin1_names = sources.admin1_names()
    lines = [f"C\t{code}\t{name}" for code, name in sorted(countries.items())]
    admin1_index: dict[str, int] = {}
    for _kind, row in places:
        key = f"{row.country_code}.{row.admin1_code}"
        if key in admin1_names and key not in admin1_index:
            admin1_index[key] = len(admin1_index)
            lines.append(f"A\t{admin1_names[key]}")
    for kind, row in places:
        admin = admin1_index.get(f"{row.country_code}.{row.admin1_code}", "")
        lines.append(f"{kind}\t{row.latitude:.4f}\t{row.longitude:.4f}\t{row.country_code}"
                     f"\t{admin}\t{row.name}\t{row.population}")
    return lines


def report(places: list[tuple[str, sources.Row]], lines: list[str]) -> None:
    counts = Counter(kind for kind, _ in places)
    by_kind = defaultdict(list)
    for line in lines:
        by_kind[line.split("\t", 1)[0]].append(line)
    for kind, count in counts.most_common():
        size = len(zlib.compress("\n".join(by_kind[kind]).encode(), 9))
        print(f"{kind:>15} {count:>7} places {size / 1e6:6.2f} MB")
    print(f"{ASSET.name}: {ASSET.stat().st_size / 1e6:.2f} MB")


def main() -> None:
    countries = sources.countries()
    places = collect_places(countries)
    lines = asset_lines(countries, places)
    ASSET.parent.mkdir(exist_ok=True)
    data = ("\n".join(lines) + "\n").encode("ascii")
    with open(ASSET, "wb") as out:
        out.write(gzip.compress(data, compresslevel=9, mtime=0))
    report(places, lines)


if __name__ == "__main__":
    main()
