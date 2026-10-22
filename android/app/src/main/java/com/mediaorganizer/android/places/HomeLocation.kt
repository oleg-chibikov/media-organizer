package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Place

/** Where a home came from. A stronger source replaces a weaker one, as on desktop. */
enum class HomeSource(internal val rank: Int) {
    TIME_ZONE(0),
    LEARNED(1),
    DEVICE(2),
    MANUAL(3),
}

/** The saved home, as the desktop app's `HomeLocation`. */
data class HomeLocation(
    val name: String,
    val country: String,
    val latitude: Double,
    val longitude: Double,
    val source: HomeSource,
) {
    internal val home: Home get() = Home(name, country)
}

internal fun homeAt(city: Place, source: HomeSource) =
    HomeLocation(city.name, city.countryName, city.latitude, city.longitude, source)

/** The home to keep when [offered] comes in: it wins only over a weaker source, as `offer_home_location`. */
fun outranking(saved: HomeLocation?, offered: HomeLocation): HomeLocation =
    if (saved == null || offered.source.rank > saved.source.rank) offered else saved
