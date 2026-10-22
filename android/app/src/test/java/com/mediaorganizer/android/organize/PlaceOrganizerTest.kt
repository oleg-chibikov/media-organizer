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
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.exists
import kotlin.io.path.writeText

class PlaceOrganizerTest {
    @get:Rule val temp = TemporaryFolder()

    private val kyoto = Place("Kyoto", PlaceKind.CITY, null, "JP", "Japan", 35.0, 135.7, 1_400_000)
    private val sydney = Place("Sydney", PlaceKind.CITY, null, "AU", "Australia", -33.9, 151.2, 5_000_000)
    private val places = PlaceSources({ Resolution(kyoto, kyoto) }, MemoryHomes(homeAt(sydney, HomeSource.LEARNED)))
    private val readMetadata = MetadataReader { _, _ ->
        RawMetadata(
            dateTimeOriginal = "2026:03:05 07:20:17",
            deviceName = "Pixel 7",
            gps = Coordinates(kyoto.latitude, kyoto.longitude),
        )
    }

    @Test fun movesIntoThePlaceFolderAndGathersEarlierDays() {
        val source = temp.root.toPath().resolve("Camera")
        val destination = temp.root.toPath().resolve("Sorted")
        write(source.resolve("IMG_1.jpg"))
        write(destination.resolve("2026/3/3/Japan/Kyoto/old.jpg"))

        val moved = organize(OrganizeRequest(source, destination), readMetadata, RunProgress(), places) { false }

        val month = destination.resolve("2026/3/Japan/Kyoto")
        assertTrue(month.resolve("5/2026-03-05 07-20-17 Kyoto, Japan (IMG_1) - Pixel 7.jpg").exists())
        assertTrue(month.resolve("3/old.jpg").exists())
        assertFalse("emptied earlier day is removed", destination.resolve("2026/3/3").exists())
        assertEquals(2, moved.size)
    }

    @Test fun keepsDateFoldersWithoutPlaces() {
        val source = temp.root.toPath().resolve("Camera")
        val destination = temp.root.toPath().resolve("Sorted")
        write(source.resolve("IMG_1.jpg"))

        organize(OrganizeRequest(source, destination), readMetadata, RunProgress()) { false }

        assertTrue(destination.resolve("2026/3/5/2026-03-05 07-20-17 (IMG_1) - Pixel 7.jpg").exists())
    }

    private fun write(path: Path) {
        path.parent.createDirectories()
        path.writeText(path.toString())
    }
}
