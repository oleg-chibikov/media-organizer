package com.mediaorganizer.android.organize

enum class FileOutcome { MOVED, DUPLICATE, NO_DATE, FAILED }

private const val LOG_LIMIT = 300

/** Counters and the newest log lines, written by the worker and read by the screen. */
class RunProgress {
    data class Snapshot(
        val total: Int,
        val counts: Map<FileOutcome, Int>,
        val skipped: Int,
        val finished: Boolean,
        val logVersion: Int,
        val log: List<String>,
    ) {
        val done: Int get() = counts.values.sum()
    }

    private val counts = FileOutcome.entries.associateWithTo(mutableMapOf()) { 0 }
    private val log = ArrayDeque<String>()
    private var logVersion = 0
    private var total = 0
    private var skipped = 0
    private var finished = false

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

    @Synchronized fun finish() {
        finished = true
    }

    @Synchronized fun snapshot() = Snapshot(total, counts.toMap(), skipped, finished, logVersion, log.toList())
}
