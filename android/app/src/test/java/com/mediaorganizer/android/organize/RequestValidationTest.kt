package com.mediaorganizer.android.organize

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class RequestValidationTest {
    @get:Rule val temp = TemporaryFolder()

    @Test fun blankDestinationIsProcessedInsideSource() {
        val source = temp.root.toPath()
        val check = checkRequest(" $source ", "") as RequestCheck.Valid
        assertEquals(OrganizeRequest(source, source.resolve("Processed")), check.request)
    }

    @Test fun rejectsRelativeMissingAndProcessedSources() {
        assertTrue(checkRequest("DCIM/Camera", "") is RequestCheck.Invalid)
        assertTrue(checkRequest("${temp.root}/missing", "") is RequestCheck.Invalid)
        val processed = temp.newFolder("Processed").path
        assertTrue(checkRequest(processed, "") is RequestCheck.Invalid)
    }

    @Test fun rejectsDestinationEqualToSource() {
        val source = temp.root.path
        assertTrue(checkRequest(source, source) is RequestCheck.Invalid)
        assertTrue(checkRequest(source, "$source/.") is RequestCheck.Invalid)
    }
}
