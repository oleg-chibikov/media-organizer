package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Place
import java.time.LocalDate

/** The city with the most distinct geotagged days. */
internal class HomeCity(val place: Place, val days: Int) {
    val home: Home get() = Home(place.name, place.countryName)

    /** Enough days to keep as home for later runs, such as one month folder. */
    val worthSaving: Boolean get() = days >= LEARNED_HOME_MIN_DAYS
}

/** The city with the most distinct geotagged days, when it has enough of them. [shots] must be in time order. */
internal fun inferHome(shots: List<Shot>, places: PlaceTable): HomeCity? {
    val allDays = HashSet<LocalDate>()
    // Per city: the last day counted and the number of distinct days.
    val cityDays = HashMap<Int, Pair<LocalDate, Int>>()
    for (shot in shots) {
        val location = shot.location ?: continue
        val day = shot.taken.toLocalDate()
        allDays += day
        val city = location.city ?: continue
        val (lastDay, days) = cityDays[city] ?: (day to 0)
        if (days == 0 || lastDay != day) cityDays[city] = day to days + 1
    }
    val (city, days) = cityDays
        .map { (city, counted) -> city to counted.second }
        .maxWithOrNull(compareBy<Pair<Int, Int>> { it.second }.thenByDescending { it.first })
        ?: return null
    val enoughShare = days * 100 >= allDays.size * HOME_MIN_DAY_SHARE_PERCENT
    if (days < HOME_MIN_DAYS || !enoughShare) return null
    return HomeCity(places[city], days)
}
