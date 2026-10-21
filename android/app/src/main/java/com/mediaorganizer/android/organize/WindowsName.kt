package com.mediaorganizer.android.organize

private val invalidWindowsChars = setOf('<', '>', ':', '"', '/', '\\', '|', '?', '*')

/** Keeps names valid on Windows so the folders copy to a PC or NAS unchanged. */
fun sanitizeForWindowsFilename(input: String): String {
    val sanitized = input
        .map { if (it in invalidWindowsChars) '_' else it }
        .joinToString("")
        .trimEnd('.', ' ')
    return sanitized.ifEmpty { "_" }
}
