package com.mediaorganizer.android.places

import org.junit.Assert.assertEquals
import org.junit.Test
import java.nio.file.Path
import java.time.LocalDate

/** The desktop cases of `src-tauri/src/planner/tests/place_levels.rs`. */
class PlaceFoldersTest {
    private fun away(place: String, country: String) = FilePlace(place, country, HomeMatch.AWAY)

    private fun plan(vararg files: Pair<String, FilePlace?>): List<String> {
        val dates = files.associate { (date, _) -> Path.of(date) to LocalDate.parse(date.take(10)) }
        val places = files.mapNotNull { (date, place) -> place?.let { Path.of(date) to it } }.toMap()
        val folders = PlaceFolders.of(places, dates)
        return files.map { (date, _) ->
            val day = LocalDate.parse(date.take(10))
            (folders.relativeDir(Path.of(date), day) ?: datedDir(day, emptyList(), false)).toString()
        }
    }

    @Test fun aCountryAndPlaceOverMonthsSitInTheYear() = assertEquals(
        listOf("2024/Japan/Kyoto/3/15", "2024/Japan/Kyoto/4/2"),
        plan("2024-03-15" to away("Kyoto", "Japan"), "2024-04-02" to away("Kyoto", "Japan")),
    )

    @Test fun placesOfOneDaySitInThatDayInsideALongerCountry() = assertEquals(
        listOf("2024/Japan/3/15/Kyoto", "2024/Japan/3/15/Osaka", "2024/Japan/4/20/Tokyo"),
        plan(
            "2024-03-15 a" to away("Kyoto", "Japan"),
            "2024-03-15 b" to away("Osaka", "Japan"),
            "2024-04-20" to away("Tokyo", "Japan"),
        ),
    )

    @Test fun aPlaceOverDaysOfOneMonthSitsInTheMonth() = assertEquals(
        listOf("2024/Japan/3/Kyoto/14", "2024/Japan/3/Kyoto/15", "2024/Japan/5/1/Tokyo"),
        plan("2024-03-14" to away("Kyoto", "Japan"), "2024-03-15" to away("Kyoto", "Japan"), "2024-05-01" to away("Tokyo", "Japan")),
    )

    @Test fun homeCountryHasNoCountryFolderAndHomeCityNoPlace() = assertEquals(
        listOf("2024/Melbourne/3/15", "2024/Melbourne/6/1", "2024/3/16", "2024/3/17"),
        plan(
            "2024-03-15" to FilePlace("Melbourne", "Australia", HomeMatch.COUNTRY),
            "2024-06-01" to FilePlace("Melbourne", "Australia", HomeMatch.COUNTRY),
            "2024-03-16" to FilePlace("Sydney", "Australia", HomeMatch.CITY),
            "2024-03-17" to null,
        ),
    )

    @Test fun yearsDontShareAFolder() = assertEquals(
        listOf("2023/12/31/Japan/Kyoto", "2024/1/1/Japan/Kyoto"),
        plan("2023-12-31" to away("Kyoto", "Japan"), "2024-01-01" to away("Kyoto", "Japan")),
    )

    @Test(expected = IllegalArgumentException::class)
    fun aPlaceNameThatIsAPathIsAnError() {
        plan("2024-03-15" to away("../Kyoto", "Japan"))
    }
}
