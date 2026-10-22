package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Place
import com.mediaorganizer.android.geo.PlaceKind
import com.mediaorganizer.android.geo.Resolution
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.nio.file.Path
import java.time.LocalDateTime

class DetectionTest {
    private val kyoto = city("Kyoto", "Japan", 35.0, 135.7)
    private val sydney = city("Sydney", "Australia", -33.9, 151.2)
    private val manly = city("Manly", "Australia", -33.8, 151.3)
    private val geocode = { point: Coordinates ->
        val place = listOf(kyoto, sydney, manly).first { it.latitude == point.latitude }
        Resolution(place, place)
    }

    private class Homes(var learned: Home? = null, val guess: Home? = null) : HomeStore {
        override fun learned() = learned
        override fun saveLearned(home: Home) { learned = home }
        override fun timeZoneGuess() = guess
    }

    @Test fun aFileWithNoGpsJoinsTheSameDeviceWithinThreeHours() {
        val places = detectFilePlaces(
            PlaceScan(
                listOf(
                    shot("a", "2024-03-15T10:00", "Pixel", kyoto),
                    shot("b", "2024-03-15T12:30", "Pixel", null),
                    shot("c", "2024-03-15T14:00", "Pixel", null),
                    shot("d", "2024-03-15T10:30", "Canon", null),
                ),
                geocode,
            ),
            Homes(guess = Home("Sydney", "Australia")),
        )
        assertEquals(FilePlace("Kyoto", "Japan", HomeMatch.AWAY), places[Path.of("a")])
        assertEquals(places[Path.of("a")], places[Path.of("b")])
        assertNull(places[Path.of("c")])
        assertNull(places[Path.of("d")])
    }

    @Test fun theTimeZoneGuessMarksHomeCityAndCountry() {
        val places = detectFilePlaces(
            PlaceScan(listOf(shot("a", "2024-03-15T10:00", null, sydney), shot("b", "2024-03-16T10:00", null, manly)), geocode),
            Homes(guess = Home("Sydney", "Australia")),
        )
        assertEquals(HomeMatch.CITY, places.getValue(Path.of("a")).home)
        assertEquals(HomeMatch.COUNTRY, places.getValue(Path.of("b")).home)
    }

    @Test fun sixtyDaysInOneCityAreLearnedAndBeatTheTimeZone() {
        val days = (0L until 60L).map { shot("d$it", LocalDateTime.of(2024, 1, 1, 9, 0).plusDays(it).toString(), null, sydney) }
        val homes = Homes(guess = Home("Kyoto", "Japan"))

        val places = detectFilePlaces(PlaceScan(days, geocode), homes)

        assertEquals(Home("Sydney", "Australia"), homes.learned)
        assertEquals(HomeMatch.CITY, places.getValue(Path.of("d0")).home)
    }

    private fun shot(name: String, taken: String, device: String?, at: Place?) =
        PlaceInput(Path.of(name), LocalDateTime.parse(taken), device, at?.let { Coordinates(it.latitude, it.longitude) })

    private fun city(name: String, country: String, latitude: Double, longitude: Double) =
        Place(name, PlaceKind.CITY, null, "XX", country, latitude, longitude, 1_000_000)
}
