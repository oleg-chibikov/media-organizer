package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Resolution

/** What place folders need from the app: a geocoder and where home is kept. */
class PlaceSources(val geocode: (Coordinates) -> Resolution, val homes: HomeStore)
