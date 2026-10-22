package com.mediaorganizer.android.location

import android.content.Context
import com.mediaorganizer.android.geo.AssetOpener
import com.mediaorganizer.android.geo.Geocoder
import com.mediaorganizer.android.places.PlaceSources

/** What a run needs for place folders, or null when the Group by place box is off. */
fun appPlaceSources(context: Context): PlaceSources? {
    if (!GroupByPlace.isOn(context)) return null
    val geocoder = AppGeocoder.of(context)
    return PlaceSources(geocoder::resolve, SavedHome(context, geocoder))
}

/** One geocoder per process, so the place tables load once while Android keeps the process. */
private object AppGeocoder {
    @Volatile private var geocoder: Geocoder? = null

    fun of(context: Context): Geocoder = geocoder ?: synchronized(this) {
        val assets = context.applicationContext.assets
        geocoder ?: Geocoder(AssetOpener(assets::open)).also { geocoder = it }
    }
}
