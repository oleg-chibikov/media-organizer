package com.mediaorganizer.android.organize

enum class FileOutcome { MOVED, DUPLICATE, NO_DATE, FAILED }

enum class RunEnd { DONE, STOPPED, FAILED }

private const val LOG_LIMIT = 300

/** Counters and the newest log lines, written by the worker and read by the screen. */
class RunProgress {
    data class Snapshot(
        val total: Int,
        val counts: Map<FileOutcome, Int>,
        val skipped: Int,
        /** Null while the run is going. */
        val end: RunEnd?,
        val logVersion: Int,
        val log: List<String>,
    ) {
        val done: Int get() = counts.values.sum()
        val finished: Boolean get() = end != null
    }

    private val counts = FileOutcome.entries.associateWithTo(mutableMapOf()) { 0 }
    private val log = ArrayDeque<String>()
    private var logVersion = 0
    private var total = 0
    private var skipped = 0
    private var end: RunEnd? = null

    @Synchronized fun scanned(total: Int, skipped: Int) {
        this.total = total
        this.skipped = skipped
    }

    @Synchronized fun record(outcome: FileOutcome, line: String) {
        counts[outcome] = counts.getValue(outcome) + 1
        log(line)
    }

    @Synchronized fun log(line: String) {
        if (log.size == LOG_LIMIT) log.removeFirst()
        log.addLast(line)
        logVersion++
    }

    @Synchronized fun finish(end: RunEnd) {
        this.end = end
    }

    @Synchronized fun snapshot() = Snapshot(total, counts.toMap(), skipped, end, logVersion, log.toList())
}
