package com.mediaorganizer.android.organize

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Place
import com.mediaorganizer.android.geo.PlaceKind
import com.mediaorganizer.android.geo.Resolution
import com.mediaorganizer.android.places.HomeSource
import com.mediaorganizer.android.places.MemoryHomes
import com.mediaorganizer.android.places.PlaceSources
import com.mediaorganizer.android.places.homeAt
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Files
import kotlin.io.path.createDirectories
import kotlin.io.path.name
import kotlin.io.path.writeText

/** The use cases of `docs/places.md`, each sorted the way the automatic moves sort: one run per batch. */
class UseCasesTest {
    @get:Rule val temp = TemporaryFolder()

    private val places = listOf(
        Place("Kyoto", PlaceKind.CITY, null, "JP", "Japan", 35.0, 135.7, 1_400_000),
        Place("Osaka", PlaceKind.CITY, null, "JP", "Japan", 34.7, 135.5, 2_700_000),
        Place("Sydney", PlaceKind.CITY, null, "AU", "Australia", -33.9, 151.2, 5_000_000),
        Place("Melbourne", PlaceKind.CITY, null, "AU", "Australia", -37.8, 145.0, 5_000_000),
    ).associateBy { it.name }
    private val sources = PlaceSources(
        { point -> places.values.first { it.latitude == point.latitude }.let { Resolution(it, it) } },
        MemoryHomes(homeAt(places.getValue("Sydney"), HomeSource.MANUAL)),
    )
    // `Kyoto_20260305_1.jpg` was shot in Kyoto on 2026-03-05.
    private val readMetadata = MetadataReader { path, _ ->
        val (place, date) = path.name.split('_')
        val city = places.getValue(place)
        RawMetadata(
            dateTimeOriginal = "${date.substring(0, 4)}:${date.substring(4, 6)}:${date.substring(6, 8)} 10:00:00",
            gps = Coordinates(city.latitude, city.longitude),
        )
    }
    private val gallery get() = temp.root.toPath().resolve("Camera")
    private val sorted get() = gallery.resolve("Processed")

    @Test fun morePhotosOnTheSameDayJoinTheDayFolder() {
        shoot("Kyoto_20260305_1.jpg")
        shoot("Kyoto_20260305_2.jpg")
        assertEquals(setOf("2026/3/5/Japan/Kyoto/Kyoto_20260305_1.jpg", "2026/3/5/Japan/Kyoto/Kyoto_20260305_2.jpg"), folders())
    }

    @Test fun aThirdDayJoinsTheMonthFolder() {
        shoot("Kyoto_20260305_1.jpg")
        shoot("Kyoto_20260306_1.jpg")
        shoot("Kyoto_20260307_1.jpg")
        assertEquals(
            setOf("2026/3/Japan/Kyoto/5/Kyoto_20260305_1.jpg", "2026/3/Japan/Kyoto/6/Kyoto_20260306_1.jpg", "2026/3/Japan/Kyoto/7/Kyoto_20260307_1.jpg"),
            folders(),
        )
    }

    @Test fun anotherCityOfTheCountryOnTheNextDayKeepsItsOwnDayFolder() {
        shoot("Kyoto_20260305_1.jpg")
        shoot("Osaka_20260306_1.jpg")
        assertEquals(setOf("2026/3/5/Japan/Kyoto/Kyoto_20260305_1.jpg", "2026/3/6/Japan/Osaka/Osaka_20260306_1.jpg"), folders())
    }

    @Test fun twoCitiesOfTheCountryInOneRunShareTheCountryMonth() {
        shoot("Kyoto_20260305_1.jpg", "Osaka_20260306_1.jpg")
        assertEquals(setOf("2026/3/Japan/5/Kyoto/Kyoto_20260305_1.jpg", "2026/3/Japan/6/Osaka/Osaka_20260306_1.jpg"), folders())
    }

    @Test fun homesCityKeepsDateFoldersAndHomesCountryGetsNoCountryFolder() {
        shoot("Sydney_20260305_1.jpg", "Melbourne_20260306_1.jpg")
        assertEquals(setOf("2026/3/5/Sydney_20260305_1.jpg", "2026/3/6/Melbourne/Melbourne_20260306_1.jpg"), folders())
    }

    @Test fun aWeekendInHomesCountryLiftsToTheMonth() {
        shoot("Melbourne_20260307_1.jpg")
        shoot("Melbourne_20260308_1.jpg")
        assertEquals(setOf("2026/3/Melbourne/7/Melbourne_20260307_1.jpg", "2026/3/Melbourne/8/Melbourne_20260308_1.jpg"), folders())
    }

    @Test fun theSamePlaceAYearLaterStartsAgain() {
        shoot("Kyoto_20250305_1.jpg")
        shoot("Kyoto_20260306_1.jpg")
        assertEquals(setOf("2025/3/5/Japan/Kyoto/Kyoto_20250305_1.jpg", "2026/3/6/Japan/Kyoto/Kyoto_20260306_1.jpg"), folders())
    }

    /** Puts photos in the gallery and sorts it once. File names keep only the original part. */
    private fun shoot(vararg names: String) {
        for (name in names) gallery.resolve(name).also { it.parent.createDirectories() }.writeText(name)
        organize(OrganizeRequest(gallery, sorted), readMetadata, RunProgress(), sources) { false }
    }

    /** Each sorted file as `<folder>/<original name>`, with the desktop stem cut back to the original. */
    private fun folders(): Set<String> = Files.walk(sorted).use { paths ->
        paths.filter(Files::isRegularFile).map { file ->
            val original = file.name.substringAfter("(").substringBefore(")") + ".jpg"
            "${sorted.relativize(file.parent)}/$original"
        }.toList().toSet()
    }
}
