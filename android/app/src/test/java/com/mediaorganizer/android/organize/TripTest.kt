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
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.exists
import kotlin.io.path.invariantSeparatorsPathString
import kotlin.io.path.name
import kotlin.io.path.readText
import kotlin.io.path.writeText

/** A trip shot day by day, sorted after each day, as the automatic moves do. Every photo must survive. */
class TripTest {
    @get:Rule val temp = TemporaryFolder()

    private val kyoto = Place("Kyoto", PlaceKind.CITY, null, "JP", "Japan", 35.0, 135.7, 1_400_000)
    private val sydney = Place("Sydney", PlaceKind.CITY, null, "AU", "Australia", -33.9, 151.2, 5_000_000)
    private val places = PlaceSources({ Resolution(kyoto, kyoto) }, MemoryHomes(homeAt(sydney, HomeSource.MANUAL)))
    // The photo's date is in its name: `IMG_0305_1.jpg` was shot on March 5.
    private val readMetadata = MetadataReader { path, _ ->
        val (month, day) = path.name.substring(4, 6) to path.name.substring(6, 8)
        RawMetadata(dateTimeOriginal = "2026:$month:$day 10:00:00", deviceName = "Pixel", gps = Coordinates(35.0, 135.7))
    }
    private val gallery get() = temp.root.toPath().resolve("Camera")
    private val sorted get() = temp.root.toPath().resolve("Sorted")

    @Test fun aSecondDayLiftsThePlaceToTheMonthAndASecondMonthToTheYear() {
        shoot("IMG_0305_1.jpg", "IMG_0305_2.jpg")
        assertEquals(setOf("2026/3/5/Japan/Kyoto/2026-03-05 10-00-00 (IMG_0305_1) - Pixel.jpg", "2026/3/5/Japan/Kyoto/2026-03-05 10-00-00 (IMG_0305_2) - Pixel.jpg"), sortedFiles())

        shoot("IMG_0306_1.jpg")
        assertEquals(
            setOf(
                "2026/3/Japan/Kyoto/5/2026-03-05 10-00-00 (IMG_0305_1) - Pixel.jpg",
                "2026/3/Japan/Kyoto/5/2026-03-05 10-00-00 (IMG_0305_2) - Pixel.jpg",
                "2026/3/Japan/Kyoto/6/2026-03-06 10-00-00 (IMG_0306_1) - Pixel.jpg",
            ),
            sortedFiles(),
        )
        assertFalse("the emptied day folder is gone", sorted.resolve("2026/3/5").exists())

        shoot("IMG_0402_1.jpg")
        assertEquals(
            setOf(
                "2026/Japan/Kyoto/3/5/2026-03-05 10-00-00 (IMG_0305_1) - Pixel.jpg",
                "2026/Japan/Kyoto/3/5/2026-03-05 10-00-00 (IMG_0305_2) - Pixel.jpg",
                "2026/Japan/Kyoto/3/6/2026-03-06 10-00-00 (IMG_0306_1) - Pixel.jpg",
                "2026/Japan/Kyoto/4/2/2026-04-02 10-00-00 (IMG_0402_1) - Pixel.jpg",
            ),
            sortedFiles(),
        )
        assertFalse("the emptied month folder is gone", sorted.resolve("2026/3").exists())
    }

    @Test fun earlierDaysKeepEveryFileWhenNamesClash() {
        val earlier = "2026/3/5/Japan/Kyoto"
        write(sorted.resolve("$earlier/a.jpg"), "first a")
        write(sorted.resolve("$earlier/b.jpg"), "same b")
        write(sorted.resolve("$earlier/notes.txt"), "notes")
        write(sorted.resolve("$earlier/.hidden.jpg"), "hidden")
        write(sorted.resolve("2026/3/Japan/Kyoto/5/a.jpg"), "second a")
        write(sorted.resolve("2026/3/Japan/Kyoto/5/b.jpg"), "same b")
        val before = everyFile()

        shoot("IMG_0306_1.jpg")

        assertEquals((before + "IMG_0306_1.jpg").sorted(), everyFile())
        assertEquals("first a", sorted.resolve("2026/3/Japan/Kyoto/5/a (1).jpg").readText())
        assertEquals("same b", sorted.resolve("$earlier/b.jpg").readText())
    }

    @Test fun stoppingAtAnyPointKeepsEveryPhoto() {
        for (stopAfter in 0..12) {
            temp.root.listFiles()?.forEach { it.deleteRecursively() }
            shoot("IMG_0305_1.jpg", "IMG_0305_2.jpg", "IMG_0305_3.jpg")
            write(gallery.resolve("IMG_0306_1.jpg"), "IMG_0306_1.jpg")
            write(gallery.resolve("IMG_0306_2.jpg"), "IMG_0306_2.jpg")
            val before = everyFile()
            var checks = 0

            organize(OrganizeRequest(gallery, sorted), readMetadata, RunProgress(), places) { checks++ >= stopAfter }

            assertEquals("stopped after $stopAfter checks", before, everyFile())
        }
    }

    /** Puts photos in the gallery, named after their content, then sorts it. */
    private fun shoot(vararg names: String) {
        for (name in names) write(gallery.resolve(name), name)
        organize(OrganizeRequest(gallery, sorted), readMetadata, RunProgress(), places) { false }
    }

    private fun sortedFiles(): Set<String> = files(sorted).map { sorted.relativize(it).invariantSeparatorsPathString }.toSet()

    /** The content of every file in the gallery and the sorted folder, to show none went missing. */
    private fun everyFile(): List<String> = (files(gallery) + files(sorted)).map { it.readText() }.sorted()

    private fun files(root: Path): List<Path> =
        if (!root.exists()) emptyList() else Files.walk(root).use { paths -> paths.filter(Files::isRegularFile).toList() }

    private fun write(path: Path, text: String) {
        path.parent.createDirectories()
        path.writeText(text)
    }
}
