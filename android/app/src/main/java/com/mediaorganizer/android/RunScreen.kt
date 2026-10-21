package com.mediaorganizer.android

import android.app.Activity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.ProgressBar
import android.widget.ScrollView
import android.widget.TextView
import com.mediaorganizer.android.organize.FileOutcome
import com.mediaorganizer.android.organize.RunProgress

/** The views of the main screen and how a progress snapshot shows on them. */
class RunScreen(activity: Activity) {
    val source: EditText = activity.findViewById(R.id.source_path)
    val destination: EditText = activity.findViewById(R.id.destination_path)
    val pickSource: Button = activity.findViewById(R.id.pick_source)
    val pickDestination: Button = activity.findViewById(R.id.pick_destination)
    val start: Button = activity.findViewById(R.id.start)
    private val status: TextView = activity.findViewById(R.id.status)
    private val progressBar: ProgressBar = activity.findViewById(R.id.progress)
    private val counts: TextView = activity.findViewById(R.id.counts)
    private val logScroll: ScrollView = activity.findViewById(R.id.log_scroll)
    private val log: TextView = activity.findViewById(R.id.log)
    private var shownLogVersion = -1

    fun setRunning(running: Boolean) {
        start.setText(if (running) R.string.stop else R.string.start)
        source.isEnabled = !running
        destination.isEnabled = !running
        pickSource.isEnabled = !running
        pickDestination.isEnabled = !running
    }

    fun show(snapshot: RunProgress.Snapshot) {
        val statusLine = statusText(status.resources, snapshot)
        if (status.text.toString() != statusLine) status.text = statusLine
        status.visibility = View.VISIBLE
        progressBar.max = snapshot.total
        progressBar.progress = snapshot.done
        counts.text = counts.context.getString(
            R.string.counts,
            snapshot.done,
            snapshot.total,
            snapshot.counts.getValue(FileOutcome.MOVED),
            snapshot.counts.getValue(FileOutcome.DUPLICATE),
            snapshot.counts.getValue(FileOutcome.NO_DATE),
            snapshot.counts.getValue(FileOutcome.FAILED),
            snapshot.skipped,
        )
        if (snapshot.logVersion != shownLogVersion) {
            shownLogVersion = snapshot.logVersion
            log.text = snapshot.log.joinToString("\n")
            logScroll.post { logScroll.scrollTo(0, log.bottom) }
        }
    }
}
