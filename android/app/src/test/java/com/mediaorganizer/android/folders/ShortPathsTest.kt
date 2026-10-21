package com.mediaorganizer.android.folders

import org.junit.Assert.assertEquals
import org.junit.Test

private const val ROOT = "/storage/emulated/0"

class ShortPathsTest {
    @Test fun internalStorageShowsAsHome() {
        assertEquals("~/DCIM/Camera", shortPath("$ROOT/DCIM/Camera", ROOT))
        assertEquals("~", shortPath(ROOT, ROOT))
    }

    @Test fun otherPathsShowInFull() {
        assertEquals("/storage/1A2B-3C4D/Photos", shortPath("/storage/1A2B-3C4D/Photos", ROOT))
        assertEquals("/storage/emulated/01/DCIM", shortPath("/storage/emulated/01/DCIM", ROOT))
    }

    @Test fun homeTurnsBackIntoFullPath() {
        assertEquals("$ROOT/DCIM/Camera", fullPath(" ~/DCIM/Camera ", ROOT))
        assertEquals(ROOT, fullPath("~", ROOT))
    }

    @Test fun typedFullPathsStay() {
        assertEquals("$ROOT/DCIM", fullPath("$ROOT/DCIM", ROOT))
        assertEquals("/storage/1A2B-3C4D", fullPath("/storage/1A2B-3C4D", ROOT))
        assertEquals("~DCIM", fullPath("~DCIM", ROOT))
    }
}
