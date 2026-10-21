package com.mediaorganizer.android.background

import android.app.job.JobInfo
import android.app.job.JobScheduler
import android.content.ComponentName
import android.content.Context
import android.provider.MediaStore
import com.mediaorganizer.android.organize.SETTLE_TIME
import java.util.concurrent.TimeUnit

const val NEW_MEDIA_JOB = 1
const val DAILY_JOB = 2
const val SETTLED_JOB = 3

/**
 * The background sort: Android starts the app when the gallery changes, and once a day as a fallback.
 * No process stays alive between the jobs.
 */
object BackgroundSorting {
    private const val ENABLED_KEY = "enabled"

    fun isEnabled(context: Context): Boolean = preferences(context).getBoolean(ENABLED_KEY, false)

    fun setEnabled(context: Context, enabled: Boolean) {
        preferences(context).edit().putBoolean(ENABLED_KEY, enabled).apply()
        if (enabled) scheduleMissing(context) else scheduler(context).cancelAll()
    }

    /** Restores the jobs after a reboot, an update or a force stop. Leaves a running job alone. */
    fun scheduleMissing(context: Context) {
        if (!isEnabled(context)) return
        val scheduler = scheduler(context)
        if (scheduler.getPendingJob(NEW_MEDIA_JOB) == null) scheduler.schedule(newMediaJob(context))
        if (scheduler.getPendingJob(DAILY_JOB) == null) scheduler.schedule(dailyJob(context))
    }

    /**
     * A gallery trigger fires once. Scheduling it again before the running one finishes keeps
     * the changes that came in during the run, so a photo taken mid-run still starts a sort.
     */
    fun watchGalleryAgain(context: Context) {
        if (isEnabled(context)) scheduler(context).schedule(newMediaJob(context))
    }

    /** One sort once files skipped as still saving have settled. A newer call moves it later. */
    fun sortAgainWhenSettled(context: Context) {
        if (!isEnabled(context)) return
        val job = JobInfo.Builder(SETTLED_JOB, service(context))
            .setMinimumLatency(SETTLE_TIME.plusSeconds(1).toMillis())
            .build()
        scheduler(context).schedule(job)
    }

    private fun newMediaJob(context: Context): JobInfo {
        val builder = JobInfo.Builder(NEW_MEDIA_JOB, service(context))
        for (uri in listOf(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, MediaStore.Video.Media.EXTERNAL_CONTENT_URI)) {
            builder.addTriggerContentUri(JobInfo.TriggerContentUri(uri, JobInfo.TriggerContentUri.FLAG_NOTIFY_FOR_DESCENDANTS))
        }
        // A burst of photos starts one sort a second after the last shot, at most 5 seconds after the first.
        return builder
            .setTriggerContentUpdateDelay(TimeUnit.SECONDS.toMillis(1))
            .setTriggerContentMaxDelay(TimeUnit.SECONDS.toMillis(5))
            .build()
    }

    private fun dailyJob(context: Context): JobInfo = JobInfo.Builder(DAILY_JOB, service(context))
        .setPeriodic(TimeUnit.DAYS.toMillis(1))
        .setPersisted(true)
        .build()

    private fun service(context: Context) = ComponentName(context, SortJobService::class.java)

    private fun scheduler(context: Context) = context.getSystemService(JobScheduler::class.java)

    private fun preferences(context: Context) = context.getSharedPreferences("background", Context.MODE_PRIVATE)
}
