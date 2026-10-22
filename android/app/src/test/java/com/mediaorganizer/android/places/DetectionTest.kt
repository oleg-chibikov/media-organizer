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

    private fun home(place: Place, source: HomeSource) = homeAt(place, source)

    @Test fun aFileWithNoGpsJoinsTheSameDeviceWithinThreeHours() {
        val places = detect(
            MemoryHomes(home(sydney, HomeSource.TIME_ZONE)),
            shot("a", "2024-03-15T10:00", "Pixel", kyoto),
            shot("b", "2024-03-15T12:30", "Pixel", null),
            shot("c", "2024-03-15T14:00", "Pixel", null),
            shot("d", "2024-03-15T10:30", "Canon", null),
        ).filePlaces
        assertEquals(FilePlace("Kyoto", "Japan", HomeMatch.AWAY), places[Path.of("a")])
        assertEquals(places[Path.of("a")], places[Path.of("b")])
        assertNull(places[Path.of("c")])
        assertNull(places[Path.of("d")])
    }

    @Test fun theTimeZoneGuessMarksHomeCityAndCountry() {
        val detection = detect(
            MemoryHomes(home(sydney, HomeSource.TIME_ZONE)),
            shot("a", "2024-03-15T10:00", null, sydney),
            shot("b", "2024-03-16T10:00", null, manly),
        )
        assertEquals(HomeMatch.CITY, detection.filePlaces.getValue(Path.of("a")).home)
        assertEquals(HomeMatch.COUNTRY, detection.filePlaces.getValue(Path.of("b")).home)
        assertEquals(HomeSource.TIME_ZONE, detection.home?.source)
    }

    @Test fun sixtyDaysInOneCityAreLearnedOverTheTimeZone() {
        val homes = MemoryHomes(home(kyoto, HomeSource.TIME_ZONE))

        val detection = detect(homes, *sixtyDaysIn(sydney))

        assertEquals(home(sydney, HomeSource.LEARNED), homes.saved())
        assertEquals(HomeMatch.CITY, detection.filePlaces.getValue(Path.of("d0")).home)
    }

    @Test fun aPickedHomeBeatsWhatThePhotosSay() {
        val homes = MemoryHomes(home(kyoto, HomeSource.MANUAL))

        val detection = detect(homes, *sixtyDaysIn(sydney))

        assertEquals(home(kyoto, HomeSource.MANUAL), homes.saved())
        assertEquals(FilePlace("Sydney", "Australia", HomeMatch.AWAY), detection.filePlaces[Path.of("d0")])
    }

    @Test fun aSmallRunUsesItsBusiestCityWithoutSavingIt() {
        val homes = MemoryHomes(null)

        val detection = detect(homes, *(0L until 14L).map { day(it, sydney) }.toTypedArray())

        assertNull(homes.saved())
        assertEquals("Sydney", detection.home?.name)
    }

    @Test fun placesThatShareANameGetTheirRegion() {
        val newYork = Place("Central Park", PlaceKind.PARK, "New York", "US", "United States", 40.78, -73.97, 0)
        val portland = Place("Central Park", PlaceKind.PARK, "Oregon", "US", "United States", 45.52, -122.68, 0)
        val geocode = { point: Coordinates -> listOf(newYork, portland).first { it.latitude == point.latitude }.let { Resolution(it, null) } }
        val files = listOf(newYork, portland).mapIndexed { index, park ->
            PlaceInput(Path.of("p$index"), LocalDateTime.parse("2024-05-0${index + 1}T10:00"), null, Coordinates(park.latitude, park.longitude))
        }

        val places = detectFilePlaces(PlaceScan(files, geocode), MemoryHomes(null)).filePlaces

        assertEquals("Central Park (New York)", places.getValue(Path.of("p0")).place)
        assertEquals("Central Park (Oregon)", places.getValue(Path.of("p1")).place)
    }

    @Test fun strongerSourcesWin() {
        val manual = home(kyoto, HomeSource.MANUAL)
        assertEquals(manual, outranking(manual, home(sydney, HomeSource.DEVICE)))
        assertEquals(home(sydney, HomeSource.DEVICE), outranking(home(kyoto, HomeSource.LEARNED), home(sydney, HomeSource.DEVICE)))
        assertEquals(home(kyoto, HomeSource.TIME_ZONE), outranking(home(kyoto, HomeSource.TIME_ZONE), home(sydney, HomeSource.TIME_ZONE)))
    }

    private fun detect(homes: HomeStore, vararg shots: PlaceInput) = detectFilePlaces(PlaceScan(shots.toList(), geocode), homes)

    private fun sixtyDaysIn(place: Place) = (0L until 60L).map { day(it, place) }.toTypedArray()

    private fun day(index: Long, place: Place) =
        shot("d$index", LocalDateTime.of(2024, 1, 1, 9, 0).plusDays(index).toString(), null, place)

    private fun shot(name: String, taken: String, device: String?, at: Place?) =
        PlaceInput(Path.of(name), LocalDateTime.parse(taken), device, at?.let { Coordinates(it.latitude, it.longitude) })

    private fun city(name: String, country: String, latitude: Double, longitude: Double) =
        Place(name, PlaceKind.CITY, null, "XX", country, latitude, longitude, 1_000_000)
}

/** A [HomeStore] in memory, with the desktop ranking. */
class MemoryHomes(private var home: HomeLocation?) : HomeStore {
    override fun saved() = home

    override fun offer(home: HomeLocation) {
        this.home = outranking(this.home, home)
    }
}
