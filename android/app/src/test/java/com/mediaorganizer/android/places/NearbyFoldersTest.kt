package com.mediaorganizer.android.places

import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Path
import java.time.LocalDate
import kotlin.io.path.createDirectories
import kotlin.io.path.writeText

/** Folders an earlier run left in the destination, as in `src-tauri/src/planner/tests/nearby_folders.rs`. */
class NearbyFoldersTest {
    @get:Rule val temp = TemporaryFolder()

    private val dubai = FilePlace("Dubai", "United Arab Emirates", HomeMatch.COUNTRY)

    private fun fit(root: Path, place: FilePlace, date: String): Pair<String, List<LibraryMove>> {
        val file = Path.of("new.jpg")
        val day = LocalDate.parse(date)
        val folders = PlaceFolders.of(mapOf(file to place), mapOf(file to day))
        val nearby = folders.gatherNearby(root)
        folders.reuseExisting(root)
        val moves = folders.libraryMoves(root, nearby) { it.toString().endsWith(".jpg") }
        return folders.relativeDir(file, day).toString() to moves
    }

    private fun write(root: Path, relative: String) = root.resolve(relative).also {
        it.parent.createDirectories()
        it.writeText(relative)
    }

    @Test fun anEarlierDayInTheMonthGathersIntoTheMonth() {
        val root = temp.root.toPath()
        val photo = write(root, "2024/3/4/Dubai/a.jpg")
        write(root, "2024/3/4/Dubai/notes.txt")
        val nested = write(root, "2024/3/4/Dubai/Edits/b.jpg")

        val (dir, moves) = fit(root, dubai, "2024-03-07")

        assertEquals("2024/3/Dubai/7", dir)
        assertEquals(
            setOf(
                LibraryMove(photo, root.resolve("2024/3/Dubai/4")),
                LibraryMove(nested, root.resolve("2024/3/Dubai/4/Edits")),
            ),
            moves.toSet(),
        )
    }

    @Test fun aMonthFolderInTheNextMonthRaisesThePlaceToTheYear() {
        val root = temp.root.toPath()
        val photo = write(root, "2024/4/Dubai/2/a.jpg")

        val (dir, moves) = fit(root, dubai, "2024-03-30")

        assertEquals("2024/Dubai/3/30", dir)
        assertEquals(listOf(LibraryMove(photo, root.resolve("2024/Dubai/4/2"))), moves)
    }

    @Test fun anExistingHigherFolderWinsWithoutMoves() {
        val root = temp.root.toPath()
        write(root, "2024/Osaka/1/2/a.jpg")

        val (dir, moves) = fit(root, FilePlace("Osaka", "Japan", HomeMatch.COUNTRY), "2024-03-15")

        assertEquals("2024/Osaka/3/15", dir)
        assertEquals(emptyList<LibraryMove>(), moves)
    }

    @Test fun anotherYearStaysApart() {
        val root = temp.root.toPath()
        write(root, "2023/3/4/Dubai/a.jpg")

        val (dir, moves) = fit(root, dubai, "2024-03-07")

        assertEquals("2024/3/7/Dubai", dir)
        assertEquals(emptyList<LibraryMove>(), moves)
    }
}
