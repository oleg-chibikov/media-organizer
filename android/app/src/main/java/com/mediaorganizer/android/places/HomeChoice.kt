package com.mediaorganizer.android.places

import java.nio.file.Path

/** Where a run reads home from, and keeps the home it learns. */
interface HomeStore {
    /** Home learned from the files of an earlier run. */
    fun learned(): Home?

    fun saveLearned(home: Home)

    /** The city the phone's time zone names. A weak guess. */
    fun timeZoneGuess(): Home?
}

/**
 * The place of each file and how much of it is home, as `home_location::detect_file_places` on
 * desktop. A run that covers enough days learns home from the files and saves it. Without a learned
 * home, a smaller run uses the time zone guess, else infers home for itself.
 */
fun detectFilePlaces(scan: PlaceScan, homes: HomeStore): Map<Path, FilePlace> =
    scan.detect(homes.learned() ?: guessHome(scan, homes))

private fun guessHome(scan: PlaceScan, homes: HomeStore): Home? {
    val city = scan.homeCity()
    if (city != null && city.worthSaving) {
        homes.saveLearned(city.home)
        return city.home
    }
    // A small run is often one holiday, so a time zone guess beats its busiest city.
    return homes.timeZoneGuess() ?: city?.home
}
