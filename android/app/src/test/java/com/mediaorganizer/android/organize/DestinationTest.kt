package com.mediaorganizer.android.organize

import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Path
import java.nio.file.Paths
import java.time.LocalDateTime
import kotlin.io.path.createDirectories
import kotlin.io.path.writeText

class DestinationTest {
    @get:Rule val temp = TemporaryFolder()

    private val date = LocalDateTime.of(2026, 3, 5, 7, 20, 17)
    private val stem = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro"

    @Test fun plansDesktopFolderAndName() {
        assertEquals(Paths.get("root/2026/3/5"), datedDirectory(Paths.get("root"), date))
        assertEquals(stem, destinationStem(date, "IMG_1001", "iPhone 15 Pro"))
        assertEquals("2026-03-05 07-20-17 (IMG_1001)", destinationStem(date, "IMG_1001", null))
    }

    @Test fun sanitizesDeviceName() =
        assertEquals("2026-03-05 07-20-17 (a) - A_B", destinationStem(date, "a", "A/B"))

    @Test fun addsSuffixWhenNameIsTaken() {
        val (source, directory) = sourceAndExisting("source-bytes", "existing")
        val destination = allocateDestination(source, directory, stem, "mp4")
        assertEquals(Destination.Free(directory.resolve("$stem (1).mp4")), destination)
    }

    @Test fun keepsSourceWhenSameBytesAreThere() {
        val (source, directory) = sourceAndExisting("same", "same")
        val destination = allocateDestination(source, directory, stem, "mp4")
        assertEquals(Destination.Duplicate(directory.resolve("$stem.mp4")), destination)
    }

    @Test fun keepsSourceWhenSuffixedCopyHasSameBytes() {
        val (source, directory) = sourceAndExisting("copied", "other")
        directory.resolve("$stem (1).mp4").writeText("copied")
        val destination = allocateDestination(source, directory, stem, "mp4")
        assertEquals(Destination.Duplicate(directory.resolve("$stem (1).mp4")), destination)
    }

    private fun sourceAndExisting(sourceText: String, existingText: String): Pair<Path, Path> {
        val root = temp.root.toPath()
        val source = root.resolve("IMG_1001.mp4").also { it.writeText(sourceText) }
        val directory = root.resolve("2026/3/5").createDirectories()
        directory.resolve("$stem.mp4").writeText(existingText)
        return source to directory
    }
}
