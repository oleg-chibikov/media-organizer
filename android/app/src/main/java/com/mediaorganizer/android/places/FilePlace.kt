package com.mediaorganizer.android.places

/** How much of a file's place is home. */
enum class HomeMatch { CITY, COUNTRY, AWAY }

/** The place folder of one file, as the desktop app's `FilePlace`. */
data class FilePlace(val place: String, val country: String, val home: HomeMatch) {
    /** The country folder above the place, outside home's country. */
    val countryFolder: String? get() = if (home == HomeMatch.AWAY) country else null
}

/** Files in home's city get no place folders, and places in its country no country folder. */
data class Home(val city: String, val country: String)
