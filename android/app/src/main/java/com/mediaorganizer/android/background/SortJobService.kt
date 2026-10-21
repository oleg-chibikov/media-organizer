package com.mediaorganizer.android.background

import android.app.job.JobParameters
import android.app.job.JobService
import android.content.Context
import android.os.Environment
import com.mediaorganizer.android.OrganizeRun
import com.mediaorganizer.android.folders.SavedFolders
import com.mediaorganizer.android.organize.OrganizeRequest
import com.mediaorganizer.android.organize.RequestCheck
import com.mediaorganizer.android.organize.checkRequest
import java.util.concurrent.ConcurrentHashMap

/** Sorts the saved gallery folder into the saved Move to folder when Android starts a background job. */
class SortJobService : JobService() {
    private val runs = ConcurrentHashMap<Int, OrganizeRun>()

    override fun onStartJob(params: JobParameters): Boolean {
        // Busy means a sort from the screen is going. The next gallery change or the daily job catches up.
        val run = savedRequest(this)?.let { request -> OrganizeRun.startIfIdle(this, request) { finish(params) } }
        if (run == null) {
            rearm(params)
            return false
        }
        runs[params.jobId] = run
        return true
    }

    /** Android stops a job after about 10 minutes. The files left over go with the next job. */
    override fun onStopJob(params: JobParameters): Boolean {
        runs.remove(params.jobId)?.cancel()
        return false
    }

    private fun finish(params: JobParameters) {
        runs.remove(params.jobId)
        rearm(params)
        jobFinished(params, false)
    }

    private fun rearm(params: JobParameters) {
        if (params.jobId == NEW_MEDIA_JOB) BackgroundSorting.watchGalleryAgain(this)
        else BackgroundSorting.scheduleMissing(this)
    }
}

/** The folders last used on screen, or null when the app lost file access or a folder is gone. */
private fun savedRequest(context: Context): OrganizeRequest? {
    if (!Environment.isExternalStorageManager()) return null
    val folders = SavedFolders(context)
    return (checkRequest(folders.source, folders.destination) as? RequestCheck.Valid)?.request
}
