package com.mediaorganizer.android.location

import android.content.Context
import com.mediaorganizer.android.geo.Geocoder
import com.mediaorganizer.android.places.Home
import com.mediaorganizer.android.places.HomeStore
import com.mediaorganizer.android.places.cityForZone
import java.util.TimeZone

/**
 * Home kept between runs. A run with enough days learns it from the photos. Until then the city of
 * the phone's time zone stands in, worked out once per zone.
 */
class SavedHome(context: Context, private val geocoder: Geocoder) : HomeStore {
    private val preferences = context.getSharedPreferences("places", Context.MODE_PRIVATE)

    override fun learned(): Home? = home(LEARNED_CITY, LEARNED_COUNTRY)

    override fun saveLearned(home: Home) {
        preferences.edit().putString(LEARNED_CITY, home.city).putString(LEARNED_COUNTRY, home.country).apply()
    }

    override fun timeZoneGuess(): Home? {
        val zone = TimeZone.getDefault().id
        if (preferences.getString(ZONE, null) == zone) return home(ZONE_CITY, ZONE_COUNTRY)
        val city = cityForZone(zone, regionOf(zone), geocoder::searchCities)
        preferences.edit()
            .putString(ZONE, zone)
            .putString(ZONE_CITY, city?.name)
            .putString(ZONE_COUNTRY, city?.countryName)
            .apply()
        return city?.let { Home(it.name, it.countryName) }
    }

    private fun home(cityKey: String, countryKey: String): Home? {
        val city = preferences.getString(cityKey, null) ?: return null
        val country = preferences.getString(countryKey, null) ?: return null
        return Home(city, country)
    }

    private companion object {
        const val LEARNED_CITY = "learned_home_city"
        const val LEARNED_COUNTRY = "learned_home_country"
        const val ZONE = "time_zone"
        const val ZONE_CITY = "time_zone_city"
        const val ZONE_COUNTRY = "time_zone_country"
    }
}

/** The country code of a zone, such as `AU` for `Australia/Sydney`. `001` means no country. */
private fun regionOf(zone: String): String? = try {
    android.icu.util.TimeZone.getRegion(zone).takeIf { it != "001" }
} catch (_: IllegalArgumentException) {
    null
}
