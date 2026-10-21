package com.mediaorganizer.android.organize

import org.junit.Assert.assertEquals
import org.junit.Test

/** The same cases as `src-tauri/src/filename/tests.rs`. */
class OriginalNameTest {
    private fun assertOriginal(fileName: String, expected: String) =
        assertEquals(expected, parseOriginalName(fileName))

    @Test fun parsesParenthesizedOriginalName() =
        assertOriginal("2024-11-02 13-09-41 (PXL_20241102_020933409.LS).mp4", "PXL_20241102_020933409.LS")

    @Test fun fallsBackToStemWithoutParentheses() =
        assertOriginal("PXL_20260305_072017616.LS.mp4", "PXL_20260305_072017616.LS")

    @Test fun usesLastGroupOfSeveral() =
        assertOriginal("2026-03-05 07-20-17 (tmp) (IMG_1234).mp4", "IMG_1234")

    @Test fun ignoresTrailingCopyCounter() = assertOriginal("IMG_1234 (1).jpg", "IMG_1234")

    @Test fun keepsEmbeddedNameWithCopyCounter() =
        assertOriginal("2024-11-02 13-09-41 (PXL_20241102_020933409.LS) (2).mp4", "PXL_20241102_020933409.LS")

    @Test fun keepsNumberGroupWithoutSpace() = assertOriginal("120260101_094503(0).jpg", "120260101_094503(0)")

    @Test fun parsesNameBeforeDeviceSuffix() =
        assertOriginal("2026-01-01 09-45-05 (20260101_094504) - SM-G991B.jpg", "20260101_094504")

    @Test fun keepsNestedCounterInsideName() =
        assertOriginal("2025-03-02 15-15-16 (20250302_151515(0)) - SM-G991B.jpg", "20250302_151515(0)")

    @Test fun parsesMostNestedName() = assertOriginal(
        "2024-10-20 18-14-18 (2024-10-20 18-14-20 (PXL_20241020_071418278)) - Pixel 7 Pro.jpg",
        "PXL_20241020_071418278",
    )

    @Test fun sanitizesInvalidWindowsCharacters() {
        assertEquals("A_B_C_foo_", sanitizeForWindowsFilename("A<B>C:foo?"))
        assertEquals("name", sanitizeForWindowsFilename("name. "))
        assertEquals("_", sanitizeForWindowsFilename(""))
    }
}
