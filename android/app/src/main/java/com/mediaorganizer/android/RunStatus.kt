package com.mediaorganizer.android

import android.content.res.Resources
import com.mediaorganizer.android.organize.FileOutcome
import com.mediaorganizer.android.organize.RunEnd
import com.mediaorganizer.android.organize.RunProgress

/** The one line above the progress bar that says whether the run is going, done or stopped. */
fun statusText(resources: Resources, snapshot: RunProgress.Snapshot): String = when (snapshot.end) {
    null -> resources.getString(R.string.status_running)
    RunEnd.DONE -> snapshot.counts.getValue(FileOutcome.MOVED)
        .let { moved -> resources.getQuantityString(R.plurals.status_done, moved, moved) }
    RunEnd.STOPPED -> resources.getString(R.string.status_stopped)
    RunEnd.FAILED -> resources.getString(R.string.status_failed)
}
