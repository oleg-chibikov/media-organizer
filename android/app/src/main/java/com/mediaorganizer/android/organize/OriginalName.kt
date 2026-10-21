package com.mediaorganizer.android.organize

/**
 * Recovers the camera's original name from `<date> (<original>)[ - device][ (n)].<ext>`,
 * falling back to the stem when no original-name group exists.
 */
fun parseOriginalName(fileName: String): String {
    val stem = stripTrailingDuplicateSuffix(fileName.substringBeforeLast('.'))
    val end = stem.lastIndexOf(')')
    val start = if (end >= 0) findMatchingOpenParen(stem, end) else null
    if (start == null || start >= end) return stem

    val inner = stem.substring(start + 1, end).trim()
    if (inner.isEmpty() || isSmallNumericGroup(inner)) return stem
    return parseMostNestedGroup(inner) ?: inner
}

/** Drops a trailing ` (n)` copy counter such as `IMG_1234 (1)`. */
private fun stripTrailingDuplicateSuffix(stem: String): String {
    if (!stem.endsWith(')')) return stem
    val stripped = stem.dropLast(1)
    val start = stripped.lastIndexOf('(')
    if (start < 0) return stem

    val inner = stripped.substring(start + 1).trim()
    val hasSpaceBefore = start > 0 && stripped[start - 1].isWhitespace()
    return if (hasSpaceBefore && isSmallNumericGroup(inner)) stripped.substring(0, start).trimEnd() else stem
}
