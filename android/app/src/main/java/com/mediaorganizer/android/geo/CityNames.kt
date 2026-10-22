package com.mediaorganizer.android.geo

/** Folded city names, biggest city first, for finding cities by name. */
internal class CityNames(private val index: GeoIndex) {
    private val names: List<Pair<String, Int>> = (0 until index.size)
        .filter { index.kind(it) == PlaceKind.CITY }
        .sortedByDescending(index::population)
        .map { fold(index.name(it)) to it }

    /** Up to [limit] cities whose name starts with [query], then ones that contain it. Case and accents are ignored. */
    fun search(query: String, limit: Int): List<Place> {
        val folded = fold(query.trim())
        if (folded.isEmpty()) return emptyList()
        val starting = names.asSequence().filter { (name, _) -> name.startsWith(folded) }
        val containing = names.asSequence().filter { (name, _) -> !name.startsWith(folded) && folded in name }
        return (starting + containing).take(limit).map { (_, entry) -> index.place(entry) }.toList()
    }
}
