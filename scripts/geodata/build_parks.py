"""Builds src-tauri/assets/parks.tsv.gz from Overture Maps park outlines (ODbL).

Run from the repo root: python3 scripts/geodata/build_parks.py
Needs DuckDB and anyascii: pip install duckdb anyascii. The first run
downloads about 650 MB from Overture's public S3 bucket into
scripts/geodata/.cache and takes about 30 minutes.
"""

import gzip
import json
import urllib.request

import duckdb

from download import CACHE_DIR
from park_lines import park_line

ASSET = CACHE_DIR.parent.parents[1] / "src-tauri" / "assets" / "parks.tsv.gz"
CATALOG = "https://stac.overturemaps.org/catalog.json"
CLASSES = ("national_park", "state_park", "wilderness_area")
MIN_AREA_KM2 = 1
# About 50 m. Outlines only decide which park a photo was taken in.
SIMPLIFY_DEGREES = 0.0005


def cached_parks(con: duckdb.DuckDBPyConnection) -> str:
    """Downloads the named protected areas of the latest release once."""
    CACHE_DIR.mkdir(exist_ok=True)
    release = json.load(urllib.request.urlopen(CATALOG))["latest"]
    target = CACHE_DIR / f"overture-protected-{release}.parquet"
    if not target.exists():
        print(f"Downloading Overture {release} protected areas")
        path = f"s3://overturemaps-us-west-2/release/{release}/theme=base/type=land_use/*"
        partial = target.with_suffix(".part")
        con.execute(f"""COPY (
            SELECT names.primary AS name, names.common['en'] AS name_en, class, geometry
            FROM read_parquet('{path}', hive_partitioning=1)
            WHERE subtype = 'protected' AND class IN {CLASSES} AND names.primary IS NOT NULL
        ) TO '{partial}' (FORMAT parquet)""")
        partial.rename(target)
    return str(target)


def main() -> None:
    con = duckdb.connect()
    con.execute("INSTALL httpfs; LOAD httpfs; INSTALL spatial; LOAD spatial;"
                "SET s3_region = 'us-west-2'; SET geometry_always_xy = true;")
    rows = con.execute(f"""
        SELECT coalesce(name_en, name), class, ST_Area_Spheroid(geometry) / 1e6 AS km2,
               ST_AsGeoJSON(ST_PointOnSurface(geometry)),
               ST_AsGeoJSON(ST_SimplifyPreserveTopology(geometry, {SIMPLIFY_DEGREES}))
        FROM '{cached_parks(con)}'
        WHERE class IN {CLASSES} AND ST_GeometryType(geometry) IN ('POLYGON', 'MULTIPOLYGON')
        ORDER BY 1, 3
    """).fetchall()
    lines = sorted(filter(None, (park_line(*row) for row in rows if row[2] >= MIN_AREA_KM2)))
    with open(ASSET, "wb") as out:
        out.write(gzip.compress(("\n".join(lines) + "\n").encode("utf-8"), 9, mtime=0))
    print(f"{len(lines)} parks, {ASSET.name}: {ASSET.stat().st_size / 1e6:.2f} MB")


if __name__ == "__main__":
    main()
