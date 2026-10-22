package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.TestGeocoder
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.Path

/** The cases of `src-tauri/src/file_places/tests`, with the real place tables. */
class FilePlacesParityTest {
    private val liverpoolHome = MemoryHomes(savedHome("Liverpool", "Australia", LIVERPOOL_NSW, HomeSource.MANUAL))

    private fun placesOf(files: List<PlaceInput>, homes: HomeStore = liverpoolHome) = detect(files, homes).filePlaces

    @Test fun everyGeotaggedFileGetsItsPlaceWithAForeignCountry() = assertEquals(
        FilePlace("Frechen", "Germany", HomeMatch.AWAY),
        placesOf(burst("de", "2025-06-01 10:00:00", 1, 0, FRECHEN))[Path.of("de-0")],
    )

    @Test fun aPlaceInHomeCountryMatchesHomeCountry() = assertEquals(
        FilePlace("Sydney", "Australia", HomeMatch.COUNTRY),
        placesOf(burst("garden", "2025-06-01 10:00:00", 1, 0, ROYAL_BOTANIC_GARDEN))[Path.of("garden-0")],
    )

    @Test fun aPlaceInHomeCityMatchesHomeCity() {
        val found = placesOf(burst("park", "2025-06-01 10:00:00", 1, 0, LIVERPOOL_NSW)).getValue(Path.of("park-0"))
        assertEquals("Australia" to HomeMatch.CITY, found.country to found.home)
    }

    @Test fun withNoHomeEveryPlaceIsAway() {
        val found = placesOf(burst("park", "2025-06-01 10:00:00", 1, 0, LIVERPOOL_NSW), MemoryHomes(null))
        assertEquals(HomeMatch.AWAY, found.getValue(Path.of("park-0")).home)
    }

    @Test fun filesWithoutGpsHaveNoPlace() =
        assertTrue(placesOf(burst("bare", "2025-06-01 10:00:00", 6, 60, null)).isEmpty())

    @Test fun theCityWithMostDaysIsHome() {
        val home = inferredHome(daily("home", "2025-01-01", 20, LIVERPOOL_NSW) + daily("away", "2025-03-01", 5, FRECHEN))
        assertEquals("Liverpool", home?.name)
    }

    @Test fun tooFewDaysGiveNoHome() = assertNull(inferredHome(daily("home", "2025-01-01", 13, LIVERPOOL_NSW)))

    @Test fun aSmallShareOfDaysGivesNoHome() = assertNull(
        inferredHome(
            daily("home", "2025-01-01", 14, LIVERPOOL_NSW) +
                daily("a", "2025-02-01", 10, FRECHEN) +
                daily("b", "2025-03-01", 10, HOFSTETTEN) +
                daily("c", "2025-04-01", 10, CASOLE_D_ELSA) +
                daily("d", "2025-05-01", 10, REGGIO_EMILIA),
        ),
    )

    @Test fun aFileWithoutGpsJoinsThePlaceOfTheSameDeviceWithin3Hours() {
        val places = placesOf(
            burst("de", "2025-06-01 10:00:00", 1, 0, FRECHEN) +
                burst("near", "2025-06-01 12:30:00", 1, 0, null) +
                burst("late", "2025-06-01 13:30:00", 1, 0, null),
        )
        assertEquals(places[Path.of("de-0")], places[Path.of("near-0")])
        assertFalse(Path.of("late-0") in places)
    }

    @Test fun aFileWithoutGpsFromAnotherDeviceStaysOut() {
        val other = burst("other", "2025-06-01 10:10:00", 1, 0, null).map { it.copy(device = "Canon EOS R6") }
        val unknown = burst("unknown", "2025-06-01 10:20:00", 1, 0, null).map { it.copy(device = null) }
        val places = placesOf(burst("de", "2025-06-01 10:00:00", 1, 0, FRECHEN) + other + unknown)
        assertFalse(Path.of("other-0") in places)
        assertFalse(Path.of("unknown-0") in places)
    }

    private fun inferredHome(files: List<PlaceInput>) = PlaceScan(files, TestGeocoder.geocoder::resolve).homeCity()?.place
}
