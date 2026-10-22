package com.mediaorganizer.android.places

import java.nio.file.Path

/** Where the saved home is kept. */
interface HomeStore {
    fun saved(): HomeLocation?

    /** Saves [home] when it outranks the saved one, as `offer_home_location` on desktop. */
    fun offer(home: HomeLocation)
}

/** What detection found, and the home it went by. */
class Detection(val filePlaces: Map<Path, FilePlace>, val home: HomeLocation?)

/**
 * The place of each file and how much of it is home, as `home_location::detect_file_places` on
 * desktop. A saved home other than a time zone guess wins. A run that covers enough days learns
 * home from the files and saves it. A smaller run uses the time zone guess, else infers home.
 */
fun detectFilePlaces(scan: PlaceScan, homes: HomeStore): Detection {
    val saved = homes.saved()
    val home = if (saved != null && saved.source != HomeSource.TIME_ZONE) saved else guessHome(scan, homes, saved)
    return Detection(scan.detect(home?.home), home)
}

private fun guessHome(scan: PlaceScan, homes: HomeStore, timeZoneGuess: HomeLocation?): HomeLocation? {
    val city = scan.homeCity()
    if (city != null && city.worthSaving) {
        return homeAt(city.place, HomeSource.LEARNED).also(homes::offer)
    }
    // A small run is often one holiday, so a time zone guess beats its busiest city.
    return timeZoneGuess ?: city?.let { homeAt(it.place, HomeSource.LEARNED) }
}
