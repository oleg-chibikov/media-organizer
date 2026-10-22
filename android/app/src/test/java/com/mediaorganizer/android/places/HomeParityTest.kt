package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.TestGeocoder
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The cases of `src-tauri/src/home_location/tests`, with the real place tables. */
class HomeParityTest {
    private val geocoder = TestGeocoder.geocoder

    private class FakeLocator(val point: Coordinates? = null, val zone: String? = null) : DeviceLocator {
        var asked = 0
        override fun currentPoint(): Coordinates? = point.also { asked++ }
        override fun timeZone() = zone
        // Desktop reads these from the system `zone.tab`.
        override fun countryOfZone(zone: String) =
            mapOf("Australia/Sydney" to "AU", "America/Argentina/Buenos_Aires" to "AR")[zone]
    }

    private fun located(point: Coordinates) = FakeLocator(point, "Australia/Sydney")

    private fun named(locator: FakeLocator) = deviceHome(locator, geocoder)?.let { Triple(it.name, it.country, it.source) }

    @Test fun aSavedHomePutsATripAbroadInItsCountry() {
        val homes = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.MANUAL))
        val found = detect(daily("trip", "2025-10-01", 20, HOUSTON), homes).filePlaces
        assertTrue(found.values.all { it.countryFolder == "United States" })
        assertEquals("Sydney", homes.saved()?.name)
    }

    @Test fun aRunWith60DaysAtHomeSavesItAsLearned() {
        val homes = MemoryHomes(null)
        detect(daily("home", "2025-01-01", 60, LIVERPOOL_NSW), homes)
        val home = homes.saved()!!
        assertEquals(Triple("Liverpool", "Australia", HomeSource.LEARNED), Triple(home.name, home.country, home.source))
    }

    @Test fun aSmallerRunInfersHomeForItselfAndSavesNone() {
        val homes = MemoryHomes(null)
        val found = detect(daily("home", "2025-01-01", 59, LIVERPOOL_NSW) + daily("trip", "2025-03-10", 10, HOUSTON), homes)
        assertEquals(setOf(null, "United States"), found.filePlaces.values.map { it.countryFolder }.toSet())
        assertNull(homes.saved())
    }

    @Test fun aLargeRunKeepsTheSavedHome() {
        val homes = MemoryHomes(savedHome("Houston", "United States", HOUSTON, HomeSource.MANUAL))
        detect(daily("home", "2025-01-01", 90, LIVERPOOL_NSW), homes)
        assertEquals("Houston" to HomeSource.MANUAL, homes.saved()!!.let { it.name to it.source })
    }

    @Test fun aRunWith60DaysAtHomeReplacesATimeZoneGuess() {
        val homes = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.TIME_ZONE))
        detect(daily("home", "2025-01-01", 60, LIVERPOOL_NSW), homes)
        assertEquals("Liverpool" to HomeSource.LEARNED, homes.saved()!!.let { it.name to it.source })
    }

    @Test fun aRunWith60DaysAtHomeKeepsADeviceHome() {
        val homes = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.DEVICE))
        detect(daily("home", "2025-01-01", 60, LIVERPOOL_NSW), homes)
        assertEquals("Sydney" to HomeSource.DEVICE, homes.saved()!!.let { it.name to it.source })
    }

    @Test fun aTimeZoneGuessPutsATripAbroadInItsCountry() {
        val homes = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.TIME_ZONE))
        val found = detect(daily("trip", "2025-10-01", 20, HOUSTON), homes).filePlaces
        assertTrue(found.values.all { it.countryFolder == "United States" })
        assertEquals(HomeSource.TIME_ZONE, homes.saved()?.source)
    }

    @Test fun aLocationFixNamesTheNearestCity() =
        assertEquals(Triple("Perth", "Australia", HomeSource.DEVICE), named(located(PERTH)))

    @Test fun withoutAFixTheTimeZoneNamesHome() {
        assertEquals(Triple("Sydney", "Australia", HomeSource.TIME_ZONE), named(FakeLocator(zone = "Australia/Sydney")))
        assertEquals(
            Triple("Buenos Aires", "Argentina", HomeSource.TIME_ZONE),
            named(FakeLocator(zone = "America/Argentina/Buenos_Aires")),
        )
        assertNull(named(FakeLocator(zone = "Etc/UTC")))
        assertNull(named(FakeLocator()))
    }

    @Test fun startupSavesADeviceHomeWhenNoneIsSaved() {
        val homes = MemoryHomes(null)
        findHomeFromDevice(homes, located(PERTH), geocoder)
        assertEquals("Perth" to HomeSource.DEVICE, homes.saved()!!.let { it.name to it.source })
    }

    @Test fun startupReplacesATimeZoneGuessAndSkipsOtherHomes() {
        val guessed = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.TIME_ZONE))
        findHomeFromDevice(guessed, located(PERTH), geocoder)
        assertEquals("Perth", guessed.saved()?.name)

        val learned = MemoryHomes(savedHome("Sydney", "Australia", SYDNEY, HomeSource.LEARNED))
        val locator = located(PERTH)
        findHomeFromDevice(learned, locator, geocoder)
        assertEquals(0, locator.asked)
        assertEquals("Sydney", learned.saved()?.name)
    }

    @Test fun currentLocationWithoutAFixGivesNoHome() = assertNull(locatedHome(FakeLocator(zone = "Australia/Sydney"), geocoder))
}
