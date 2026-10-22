package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.TestGeocoder
import java.nio.file.Path
import java.time.LocalDateTime

// The points and photo sets of the desktop tests in `src-tauri/src/file_places/tests` and
// `src-tauri/src/home_location/tests`, so the Kotlin copy answers the same.

/** Resolves to a city with no point of interest around it. */
val FRECHEN = Coordinates(50.9, 6.8)
val CASOLE_D_ELSA = Coordinates(43.3, 11.0)
val REGGIO_EMILIA = Coordinates(44.7, 10.6)
val HOFSTETTEN = Coordinates(48.0, 11.0)
/** Resolves to a park, with Liverpool as its nearest city. */
val LIVERPOOL_NSW = Coordinates(-33.92, 150.92)
val ROYAL_BOTANIC_GARDEN = Coordinates(-33.8642, 151.2166)
val HOUSTON = Coordinates(29.76, -95.37)
val SYDNEY = Coordinates(-33.8679, 151.2073)
val PERTH = Coordinates(-31.95, 115.86)

/** The device of every desktop test file. */
const val IPHONE = "iPhone 15 Pro"

/** `count` files from `start`, one every `stepMinutes`, all from one iPhone. */
fun burst(prefix: String, start: String, count: Int, stepMinutes: Long, gps: Coordinates?): List<PlaceInput> =
    (0 until count).map { step ->
        val taken = LocalDateTime.parse(start.replace(' ', 'T')).plusMinutes(stepMinutes * step)
        PlaceInput(Path.of("$prefix-$step"), taken, IPHONE, gps)
    }

/** One photo a day for [days] days from [firstDay] at [point]. */
fun daily(prefix: String, firstDay: String, days: Int, point: Coordinates) =
    burst(prefix, "$firstDay 12:00:00", days, 24 * 60, point)

fun detect(files: List<PlaceInput>, homes: HomeStore): Detection =
    detectFilePlaces(PlaceScan(files, TestGeocoder.geocoder::resolve), homes)

fun savedHome(name: String, country: String, point: Coordinates, source: HomeSource) =
    HomeLocation(name, country, point.latitude, point.longitude, source)
