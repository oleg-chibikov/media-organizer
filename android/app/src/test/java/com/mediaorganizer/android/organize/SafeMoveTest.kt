package com.mediaorganizer.android.organize

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.FileAlreadyExistsException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.FileTime
import kotlin.io.path.exists
import kotlin.io.path.readText
import kotlin.io.path.writeText

class SafeMoveTest {
    @get:Rule val temp = TemporaryFolder()

    private val source: Path get() = temp.root.toPath().resolve("IMG_1.jpg")
    private val destination: Path get() = temp.root.toPath().resolve("sorted.jpg")

    /** Acts like a rename onto another volume for [source] and like a plain rename for the temp copy. */
    private val otherVolume = Rename { from, to ->
        if (from == source) throw AtomicMoveNotSupportedException(from.toString(), to.toString(), "EXDEV")
        renameIfFree(from, to)
    }

    @Test fun renamesOnOneVolume() {
        source.writeText("photo")
        moveWithoutReplacing(source, destination)
        assertFalse(source.exists())
        assertEquals("photo", destination.readText())
    }

    @Test fun keepsBothFilesWhenTheNameIsTaken() {
        source.writeText("photo")
        destination.writeText("other")
        assertThrows(FileAlreadyExistsException::class.java) { moveWithoutReplacing(source, destination) }
        assertEquals("photo", source.readText())
        assertEquals("other", destination.readText())
    }

    @Test fun copiesAcrossVolumesThenRemovesSource() {
        source.writeText("video")
        val modified = FileTime.fromMillis(1_700_000_000_000)
        Files.setLastModifiedTime(source, modified)

        moveWithoutReplacing(source, destination, otherVolume)

        assertFalse(source.exists())
        assertEquals("video", destination.readText())
        assertEquals(modified, Files.getLastModifiedTime(destination))
        assertEquals(listOf("sorted.jpg"), fileNames())
    }

    @Test fun acrossVolumesKeepsBothFilesWhenTheNameIsTaken() {
        source.writeText("video")
        val taken = Rename { from, to ->
            if (from == source) throw AtomicMoveNotSupportedException(from.toString(), to.toString(), "EXDEV")
            destination.writeText("other")
            renameIfFree(from, to)
        }
        assertThrows(FileAlreadyExistsException::class.java) { moveWithoutReplacing(source, destination, taken) }
        assertEquals("video", source.readText())
        assertEquals("other", destination.readText())
        assertEquals(listOf("IMG_1.jpg", "sorted.jpg"), fileNames())
    }

    @Test fun acrossVolumesRemovesTempCopyLeftByAKilledRun() {
        source.writeText("video")
        temp.root.toPath().resolve(".sorted.jpg.media-organizer-copying-0.tmp").writeText("vi")

        moveWithoutReplacing(source, destination, otherVolume)

        assertEquals("video", destination.readText())
        assertEquals(listOf("sorted.jpg"), fileNames())
    }

    private fun fileNames() = Files.list(temp.root.toPath()).use { files -> files.map { it.fileName.toString() }.sorted().toList() }
}
