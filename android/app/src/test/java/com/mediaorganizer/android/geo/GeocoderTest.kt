package com.mediaorganizer.android.geo

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The same points as the desktop tests in `src-tauri/src/geo/tests`, so both apps name places alike. */
class GeocoderTest {
    private val geocoder = TestGeocoder.geocoder

    private fun placeAt(latitude: Double, longitude: Double): Place =
        geocoder.resolve(Coordinates(latitude, longitude)).place ?: error("no place near $latitude, $longitude")

    @Test fun yosemiteValleyIsAYosemitePlace() {
        val place = placeAt(37.7456, -119.5936)
        assertTrue(place.toString(), "Yosemite" in place.name)
        assertNotEquals(PlaceKind.CITY, place.kind)
        assertEquals("United States", place.countryName)
    }

    @Test fun moscowCentreIsMoscowRussia() {
        val city = geocoder.resolve(Coordinates(55.7558, 37.6173)).city!!
        assertEquals("Moscow", city.name)
        assertEquals("Russia", city.countryName)
    }

    @Test fun midPacificHasNoPlace() = assertNull(geocoder.resolve(Coordinates(-30.0, -140.0)).place)

    @Test fun pointsNearThePolesAndDateLineResolve() {
        for (latitude in listOf(-90.0, -89.5, 89.5, 90.0)) {
            for (longitude in listOf(-180.0, 179.99, 180.0)) geocoder.resolve(Coordinates(latitude, longitude))
        }
    }

    @Test fun sanFranciscoSightsAreSanFrancisco() {
        val sights = listOf(37.7793 to -122.4193, 37.7544 to -122.4477, 37.8080 to -122.4177, 37.8267 to -122.4230)
        for ((latitude, longitude) in sights) assertEquals("San Francisco", placeAt(latitude, longitude).name)
    }

    @Test fun aCityWithItsOwnCentreKeepsItsName() = assertEquals("Oakland", placeAt(37.8044, -122.2712).name)

    @Test fun blueMountainsTownsAreTheNationalPark() {
        for ((latitude, longitude) in listOf(-33.7125 to 150.3119, -33.7155 to 150.3317, -33.6280 to 150.3110)) {
            val place = placeAt(latitude, longitude)
            assertEquals("Blue Mountains National Park", place.name)
            assertEquals(PlaceKind.NATIONAL_PARK, place.kind)
        }
    }

    @Test fun royalNationalParkBeachesAreThePark() {
        for ((latitude, longitude) in listOf(-34.1700 to 151.0650, -34.1370 to 151.1170)) {
            assertEquals("Royal National Park", placeAt(latitude, longitude).name)
        }
    }

    @Test fun harbourSuburbsStaySydney() {
        for ((latitude, longitude) in listOf(-33.8915 to 151.2767, -33.7970 to 151.1800, -33.7970 to 151.2880)) {
            val resolution = geocoder.resolve(Coordinates(latitude, longitude))
            assertEquals("Sydney", resolution.place?.name)
            assertEquals("Sydney", resolution.city?.name)
        }
    }

    @Test fun parisToLondonIsAbout344Km() {
        val distance = distanceKm(Coordinates(48.8566, 2.3522), Coordinates(51.5074, -0.1278))
        assertEquals(344.0, distance, 2.0)
    }

    @Test fun citySearchIgnoresCaseAndAccents() {
        for (query in listOf("zür", "ZUR", "zuerich")) {
            val first = geocoder.searchCities(query, 5).first()
            assertEquals(query, "Zuerich", first.name)
            assertEquals("Switzerland", first.countryName)
        }
    }

    @Test fun citySearchReturnsAtMostTheLimit() {
        assertEquals(10, geocoder.searchCities("san", 10).size)
        assertTrue(geocoder.searchCities("  ", 10).isEmpty())
    }
}
