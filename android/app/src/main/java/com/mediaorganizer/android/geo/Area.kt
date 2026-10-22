package com.mediaorganizer.android.geo

import com.mediaorganizer.android.geo.parks.Park
import com.mediaorganizer.android.geo.parks.Parks

/** Cities this big keep their name over small parks inside them. */
private const val BIG_CITY_MIN_POPULATION = 50_000
/** Inside a big city, a park this large still wins, as Royal National Park does in Sydney. */
private const val LARGE_PARK_MIN_KM2 = 100f
/** Away from big cities, a town this close to a large park takes its name, as Katoomba takes Blue Mountains. */
private const val TOWN_PULL_KM = 5.0

/** The park or city that covers the point, named as a place. */
internal fun parkOrCityPlace(nearby: Nearby, parks: Parks): Place? {
    val point = nearby.point
    val inBigCity = nearby.coveringCities().any { (city, _) -> nearby.index.population(city) >= BIG_CITY_MIN_POPULATION }
    val park = parks.containing(point)?.takeIf { !inBigCity || it.isLarge }
        ?: parks.near(point, TOWN_PULL_KM).filter { (park, _) -> park.isLarge }.minByOrNull { it.second }?.first
            ?.takeIf { !inBigCity }
    if (park != null) return parkPlace(nearby.index, park)
    return coveringCity(nearby)?.let(nearby.index::place)
}

/** The city whose reach covers the point most deeply. A suburb wins only close to its own centre. */
internal fun coveringCity(nearby: Nearby): Int? = nearby.coveringCities().minByOrNull { it.second }?.first

/** A park as a place. Its country and region come from the GeoNames place nearest its label point. */
private fun parkPlace(index: GeoIndex, park: Park): Place? {
    // Most parks have a place within the small radius, which is quicker to search.
    val near = index.nearest(park.label, CITY_RADIUS_KM) ?: index.nearest(park.label, FALLBACK_RADIUS_KM) ?: return null
    return index.place(near).copy(
        name = park.name,
        kind = park.kind,
        latitude = park.label.latitude,
        longitude = park.label.longitude,
        population = 0,
    )
}

private val Park.isLarge: Boolean get() = areaKm2 >= LARGE_PARK_MIN_KM2
