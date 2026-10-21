package com.mediaorganizer.android.organize

private const val COPY_COUNTER_LIMIT = 100

internal fun findMatchingOpenParen(value: String, closeIndex: Int): Int? {
    if (closeIndex !in value.indices || value[closeIndex] != ')') return null
    var depth = 0
    for (index in closeIndex downTo 0) {
        when (value[index]) {
            ')' -> depth++
            '(' -> {
                if (depth == 0) return null
                depth--
                if (depth == 0) return index
            }
        }
    }
    return null
}

/** Copy counters like `(1)` or `(42)` are not original names. */
internal fun isSmallNumericGroup(value: String): Boolean =
    value.isNotEmpty() && value.all { it in '0'..'9' } &&
        (value.toIntOrNull()?.let { it < COPY_COUNTER_LIMIT } ?: false)

/** Returns the deepest non-counter group, preferring the last one at equal depth. */
internal fun parseMostNestedGroup(value: String): String? {
    val openIndexes = ArrayDeque<Int>()
    var best: String? = null
    var bestDepth = 0
    var bestIndex = -1

    value.forEachIndexed { index, char ->
        if (char == '(') {
            openIndexes.addLast(index)
            return@forEachIndexed
        }
        if (char != ')') return@forEachIndexed
        val start = openIndexes.removeLastOrNull() ?: return@forEachIndexed

        val depth = openIndexes.size + 1
        val inner = value.substring(start + 1, index).trim()
        if (inner.isEmpty() || isSmallNumericGroup(inner)) return@forEachIndexed
        if (best == null || depth > bestDepth || (depth == bestDepth && index > bestIndex)) {
            best = inner
            bestDepth = depth
            bestIndex = index
        }
    }
    return best
}
