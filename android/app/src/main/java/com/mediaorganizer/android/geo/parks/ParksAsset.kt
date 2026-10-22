package com.mediaorganizer.android.geo.parks

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.PlaceKind
import com.mediaorganizer.android.geo.tableReader
import java.io.InputStream

/**
 * Parses the park outlines that `scripts/geodata/build_parks.py` builds. Each line is tab
 * separated: `kind name area_km2 lat lon rings`. Rings are polyline strings separated by spaces.
 */
internal fun parseParks(table: InputStream): Parks {
    val parks = mutableListOf<Park>()
    tableReader(table).useLines { lines ->
        for (line in lines) {
            if (line.isEmpty()) continue
            parks += parsePark(line) ?: error("bad parks line: ${line.take(80)}")
        }
    }
    return Parks(parks)
}

private fun parsePark(line: String): Park? {
    val fields = line.split('\t')
    if (fields.size < 6) return null
    val rings = fields[5].split(' ').map { decodePolyline(it) ?: return null }
    return Park(
        name = fields[1],
        kind = PlaceKind.fromTag(fields[0]) ?: return null,
        areaKm2 = fields[2].toFloatOrNull() ?: return null,
        label = Coordinates(fields[3].toDoubleOrNull() ?: return null, fields[4].toDoubleOrNull() ?: return null),
        outline = Outline.of(rings) ?: return null,
    )
}
