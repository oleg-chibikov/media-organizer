package com.mediaorganizer.android.location

import android.content.Context
import com.mediaorganizer.android.places.HomeLocation
import com.mediaorganizer.android.places.HomeSource
import com.mediaorganizer.android.places.HomeStore
import com.mediaorganizer.android.places.outranking

/** The home kept between launches, with where it came from. The screen and background runs share it. */
class SavedHome(context: Context) : HomeStore {
    private val preferences = context.applicationContext.getSharedPreferences("home", Context.MODE_PRIVATE)

    override fun saved(): HomeLocation? = synchronized(lock) {
        val source = preferences.getString(SOURCE, null)?.let { runCatching { HomeSource.valueOf(it) }.getOrNull() }
        HomeLocation(
            name = preferences.getString(NAME, null) ?: return null,
            country = preferences.getString(COUNTRY, null) ?: return null,
            latitude = preferences.getFloat(LATITUDE, 0f).toDouble(),
            longitude = preferences.getFloat(LONGITUDE, 0f).toDouble(),
            source = source ?: return null,
        )
    }

    override fun offer(home: HomeLocation) = synchronized(lock) { replace(outranking(saved(), home)) }

    /** Saves [home] as it is, or clears it, because the person asked for it. */
    fun replace(home: HomeLocation?) = synchronized(lock) {
        val editor = preferences.edit().clear()
        if (home != null) {
            editor.putString(NAME, home.name)
                .putString(COUNTRY, home.country)
                .putFloat(LATITUDE, home.latitude.toFloat())
                .putFloat(LONGITUDE, home.longitude.toFloat())
                .putString(SOURCE, home.source.name)
        }
        editor.commit()
        Unit
    }

    private companion object {
        val lock = Any()
        const val NAME = "name"
        const val COUNTRY = "country"
        const val LATITUDE = "latitude"
        const val LONGITUDE = "longitude"
        const val SOURCE = "source"
    }
}
