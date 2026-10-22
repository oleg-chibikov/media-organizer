package com.mediaorganizer.android.geo

/** The places within 50 km of a point, gathered once and shared by every lookup. */
internal class Nearby(val index: GeoIndex, val point: Coordinates) {
    private val hits = index.near(point, CITY_RADIUS_KM)

    /** Cities whose reach covers the point, with how deep inside it sits: 0 at the centre, 1 at the edge. */
    fun coveringCities(): List<Pair<Int, Double>> = hits
        .filter { index.kind(it.entry) == PlaceKind.CITY }
        .map { it.entry to it.distance / cityReachKm(index.population(it.entry)) }
        .filter { (_, depth) -> depth <= 1.0 }

    /**
     * The most nested place whose radius covers the point wins, so a garden beats the park around it.
     * On a tie, the closest relative to its radius wins.
     */
    fun matchingPoi(): Int? = hits
        .mapNotNull { hit ->
            val kind = index.kind(hit.entry)
            val radius = kind.matchRadiusKm ?: return@mapNotNull null
            if (hit.distance <= radius) Triple(hit.entry, kind.nestingLevel, hit.distance / radius) else null
        }
        .minWithOrNull(compareBy<Triple<Int, Int, Double>> { it.second }.thenBy { it.third })
        ?.first

    /** The nearest city within 50 km. */
    fun nearestCity(): Int? = hits
        .filter { index.kind(it.entry) == PlaceKind.CITY }
        .minByOrNull(Hit::distance)
        ?.entry
}
