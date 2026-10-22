package com.mediaorganizer.android.places

import com.mediaorganizer.android.organize.isScanned
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.nio.file.Path
import java.time.LocalDate
import kotlin.io.path.createDirectories
import kotlin.io.path.writeText

/**
 * The cases of `src-tauri/src/planner/tests/nearby_folders.rs` and `existing_folders.rs`: folders an
 * earlier run left in the destination, in the same month, the months next to it, or higher up.
 */
class NearbyFoldersTest {
    @get:Rule val temp = TemporaryFolder()

    private val root: Path get() = temp.root.toPath()

    private fun home(place: String) = FilePlace(place, "Australia", HomeMatch.COUNTRY)

    private fun away(place: String, country: String) = FilePlace(place, country, HomeMatch.AWAY)

    /** The folder of each scan file and each library move as `from -> to`, relative to [root]. */
    private fun plan(
        vararg files: Pair<String, FilePlace>,
        selected: Path = Path.of("/elsewhere"),
        gather: Boolean = true,
    ): Pair<List<String>, List<String>> {
        val paths = files.indices.map { Path.of("scan-$it.jpg") }
        val dates = paths.zip(files.map { LocalDate.parse(it.first) }).toMap()
        val folders = PlaceFolders.of(paths.zip(files.map { it.second }).toMap(), dates)
        val nearby = if (gather) folders.gatherNearby(root) else emptyList()
        folders.reuseExisting(root)
        val moves = folders.libraryMoves(root, nearby) { it.toString().endsWith(".jpg") && !isScanned(it, selected, root) }
        val dirs = paths.map { folders.relativeDir(it, dates.getValue(it)).toString() }
        return dirs to moves.map { "${root.relativize(it.from)} -> ${root.relativize(it.toDirectory.resolve(it.from.fileName))}" }
    }

    private fun photoAt(relative: String) = root.resolve(relative).also {
        it.parent.createDirectories()
        it.writeText("photo")
    }

    @Test fun anEarlierDayOfTheMonthMovesIntoTheMonthFolder() {
        photoAt("2024/3/4/Dubai/a.jpg")
        assertEquals(
            listOf("2024/3/Dubai/7") to listOf("2024/3/4/Dubai/a.jpg -> 2024/3/Dubai/4/a.jpg"),
            plan("2024-03-07" to home("Dubai")),
        )
    }

    @Test fun aDayOfTheNextMonthRaisesThePlaceToItsYear() {
        photoAt("2024/2/Dubai/27/a.jpg")
        assertEquals(
            listOf("2024/Dubai/3/2") to listOf("2024/2/Dubai/27/a.jpg -> 2024/Dubai/2/27/a.jpg"),
            plan("2024-03-02" to home("Dubai")),
        )
    }

    @Test fun aDayTwoMonthsAwayStaysApart() {
        photoAt("2024/1/5/Dubai/a.jpg")
        assertEquals(listOf("2024/3/7/Dubai") to emptyList<String>(), plan("2024-03-07" to home("Dubai")))
    }

    @Test fun december31AndJanuary1StayInTheirOwnYears() {
        photoAt("2023/12/31/Dubai/a.jpg")
        assertEquals(listOf("2024/1/1/Dubai") to emptyList<String>(), plan("2024-01-01" to home("Dubai")))
    }

    @Test fun theCountryFolderMovesUpWithItsPlace() {
        photoAt("2024/3/4/UAE/Dubai/a.jpg")
        assertEquals(
            listOf("2024/3/UAE/Dubai/7") to listOf("2024/3/4/UAE/Dubai/a.jpg -> 2024/3/UAE/Dubai/4/a.jpg"),
            plan("2024-03-07" to away("Dubai", "UAE")),
        )
    }

    @Test fun filesInTheScannedFolderAndOtherFilesStayOutOfTheLibraryMoves() {
        photoAt("2024/3/4/Dubai/a.jpg")
        photoAt("2024/3/4/Dubai/notes.txt")
        photoAt("2024/3/5/Dubai/b.jpg")
        val (_, moves) = plan("2024-03-07" to home("Dubai"), selected = root.resolve("2024/3/5"))
        assertEquals(listOf("2024/3/4/Dubai/a.jpg -> 2024/3/Dubai/4/a.jpg"), moves)
    }

    @Test fun earlierDaysInTheProcessedFolderOfTheScannedFolderMove() {
        val gallery = root
        val sorted = root.resolve("Processed")
        sorted.resolve("2024/3/4/Dubai/a.jpg").also { it.parent.createDirectories() }.writeText("photo")
        val paths = Path.of("scan.jpg")
        val folders = PlaceFolders.of(mapOf(paths to home("Dubai")), mapOf(paths to LocalDate.parse("2024-03-07")))
        val nearby = folders.gatherNearby(sorted)
        folders.reuseExisting(sorted)
        val moves = folders.libraryMoves(sorted, nearby) { !isScanned(it, gallery, sorted) }
        assertEquals(listOf(sorted.resolve("2024/3/Dubai/4")), moves.map { it.toDirectory })
    }

    @Test fun subfoldersOfAnEarlierDayComeAlong() {
        photoAt("2024/3/4/Dubai/Edits/b.jpg")
        assertEquals(
            listOf("2024/3/4/Dubai/Edits/b.jpg -> 2024/3/Dubai/4/Edits/b.jpg"),
            plan("2024-03-07" to home("Dubai")).second,
        )
    }

    @Test fun oneDayJoinsThePlaceFolderOfItsMonth() {
        root.resolve("2024/3/Osaka/14").createDirectories()
        assertEquals(listOf("2024/3/Osaka/15"), plan("2024-03-15" to home("Osaka"), gather = false).first)
    }

    @Test fun oneDayJoinsThePlaceFolderOfItsYear() {
        root.resolve("2024/Osaka/1/2").createDirectories()
        assertEquals(listOf("2024/Osaka/3/15"), plan("2024-03-15" to home("Osaka"), gather = false).first)
    }

    @Test fun aPlaceJoinsTheExistingCountryAndThenItsOwnFolder() {
        root.resolve("2024/Japan/Kyoto").createDirectories()
        assertEquals(listOf("2024/Japan/Kyoto/3/15"), plan("2024-03-15" to away("Kyoto", "Japan"), gather = false).first)
        assertEquals(listOf("2024/Japan/3/15/Osaka"), plan("2024-03-15" to away("Osaka", "Japan"), gather = false).first)
    }

    @Test fun aDeeperFolderOfTheSamePlaceIsLeftAlone() {
        root.resolve("2024/3/1/Osaka").createDirectories()
        assertEquals(
            listOf("2024/3/Osaka/15", "2024/3/Osaka/16"),
            plan("2024-03-15" to home("Osaka"), "2024-03-16" to home("Osaka"), gather = false).first,
        )
    }
}
