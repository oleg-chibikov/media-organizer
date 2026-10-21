package com.mediaorganizer.android.organize

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

class OrganizerTest {
    @get:Rule val temp = TemporaryFolder()

    private val readMetadata = MetadataReader { _, _ ->
        RawMetadata(dateTimeOriginal = "2026:03:05 07:20:17", deviceName = "Pixel 7")
    }

    @Test fun movesMediaAndLeavesTheRest() {
        val source = temp.root.toPath()
        val destination = source.resolve("Processed")
        write(source.resolve("Trip/IMG_1.jpg"), "one")
        write(source.resolve("notes.txt"), "text")
        write(source.resolve(".trashed-1-IMG_2.jpg"), "trash")
        write(destination.resolve("old/IMG_3.jpg"), "organized")

        val progress = RunProgress()
        val moved = organize(OrganizeRequest(source, destination), readMetadata, progress) { false }

        val target = destination.resolve("2026/3/5/2026-03-05 07-20-17 (IMG_1) - Pixel 7.jpg")
        assertEquals(listOf(MovedFile(source.resolve("Trip/IMG_1.jpg"), target)), moved)
        assertTrue(target.exists())
        assertFalse("emptied source folder is removed", source.resolve("Trip").exists())
        assertTrue(source.resolve("notes.txt").exists())
        assertTrue(source.resolve(".trashed-1-IMG_2.jpg").exists())

        val snapshot = progress.snapshot()
        assertEquals(1, snapshot.total)
        assertEquals(1, snapshot.counts[FileOutcome.MOVED])
        assertEquals(2, snapshot.skipped)
        assertEquals(RunEnd.DONE, snapshot.end)
    }

    @Test fun keepsDuplicateInPlace() {
        val source = temp.root.toPath().resolve("Camera")
        val destination = temp.root.toPath().resolve("Sorted")
        write(source.resolve("IMG_1.jpg"), "same")
        write(destination.resolve("2026/3/5/2026-03-05 07-20-17 (IMG_1) - Pixel 7.jpg"), "same")

        val progress = RunProgress()
        organize(OrganizeRequest(source, destination), readMetadata, progress) { false }

        assertTrue(source.resolve("IMG_1.jpg").exists())
        assertEquals(1, progress.snapshot().counts[FileOutcome.DUPLICATE])
    }

    @Test fun stopsWhenCancelled() {
        val source = temp.root.toPath()
        write(source.resolve("IMG_1.jpg"), "one")

        val progress = RunProgress()
        val moved = organize(OrganizeRequest(source, source.resolve("Processed")), readMetadata, progress) { true }

        assertTrue(moved.isEmpty())
        assertTrue(source.resolve("IMG_1.jpg").exists())
        assertEquals(RunEnd.STOPPED, progress.snapshot().end)
        assertEquals("Stopped", progress.snapshot().log.last())
    }

    private fun write(path: Path, text: String) {
        path.parent.createDirectories()
        path.writeText(text)
    }
}
