package com.mediaorganizer.android

import android.content.Context
import android.media.MediaScannerConnection
import com.mediaorganizer.android.metadata.DeviceMetadataReader
import com.mediaorganizer.android.organize.MovedFile
import com.mediaorganizer.android.organize.OrganizeRequest
import com.mediaorganizer.android.organize.RunProgress
import com.mediaorganizer.android.organize.organize
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/** The organize job of this process. A recreated screen finds it here and keeps showing it. */
object ActiveRun {
    var current: OrganizeRun? = null
        private set

    fun start(context: Context, request: OrganizeRequest) {
        current = OrganizeRun(context.applicationContext, request).also(OrganizeRun::start)
    }
}

class OrganizeRun(private val context: Context, private val request: OrganizeRequest) {
    val progress = RunProgress()
    private val cancelled = AtomicBoolean(false)

    fun cancel() = cancelled.set(true)

    fun start() {
        thread(name = "organize") {
            try {
                notifyGallery(organize(request, DeviceMetadataReader, progress, cancelled::get))
            } catch (error: Exception) {
                progress.log("Stopped: $error")
                progress.finish()
            }
        }
    }

    /** The gallery reads its own index, so both the old and the new paths need a rescan. */
    private fun notifyGallery(moved: List<MovedFile>) {
        if (moved.isEmpty()) return
        val paths = moved.flatMap { listOf(it.from.toString(), it.to.toString()) }
        MediaScannerConnection.scanFile(context, paths.toTypedArray(), null, null)
    }
}
