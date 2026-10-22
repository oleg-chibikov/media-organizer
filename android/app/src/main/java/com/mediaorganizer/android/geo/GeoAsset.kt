package com.mediaorganizer.android.geo

import java.io.InputStream

/**
 * Parses the GeoNames extract that `scripts/geodata` builds. Lines are tab separated.
 * `C code name` adds a country. `A name` adds an admin1 region. Any other line is a place:
 * `kind lat lon country_code admin1_index name population`.
 */
internal fun parseGeoIndex(table: InputStream): GeoIndex {
    val rows = mutableListOf<GeoRow>()
    val names = StringBuilder()
    val countries = mutableListOf<Pair<String, String>>()
    val countryIds = HashMap<String, Int>()
    val admin1 = mutableListOf<String>()
    tableReader(table).useLines { lines ->
        for (line in lines) {
            if (line.isEmpty()) continue
            val fields = line.split('\t')
            when (fields[0]) {
                "C" -> {
                    countryIds[fields[1]] = countries.size
                    countries += fields[1] to fields[2]
                }
                "A" -> admin1 += fields.getOrElse(1) { "" }
                else -> rows += parseRow(fields, countryIds, names) ?: error("bad geodata line: $line")
            }
        }
    }
    return GeoIndex(rows, names.toString(), countries, admin1)
}

private fun parseRow(fields: List<String>, countryIds: Map<String, Int>, names: StringBuilder): GeoRow? {
    if (fields.size < 7) return null
    val name = fields[5]
    val start = names.length
    names.append(name)
    return GeoRow(
        latitude = fields[1].toFloatOrNull() ?: return null,
        longitude = fields[2].toFloatOrNull() ?: return null,
        kind = PlaceKind.fromTag(fields[0]) ?: return null,
        country = countryIds[fields[3]] ?: return null,
        admin1 = fields[4].toIntOrNull() ?: -1,
        population = fields[6].toIntOrNull() ?: return null,
        nameStart = start,
        nameEnd = names.length,
    )
}
