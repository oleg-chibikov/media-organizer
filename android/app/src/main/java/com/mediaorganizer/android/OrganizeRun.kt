package com.mediaorganizer.android

import android.content.Context
import android.media.MediaScannerConnection
import com.mediaorganizer.android.metadata.DeviceMetadataReader
import com.mediaorganizer.android.organize.MovedFile
import com.mediaorganizer.android.organize.OrganizeRequest
import com.mediaorganizer.android.organize.RunEnd
import com.mediaorganizer.android.organize.RunProgress
import com.mediaorganizer.android.organize.organize
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/** The organize job started from the screen. A recreated screen finds it here and keeps showing it. */
object ActiveRun {
    var current: OrganizeRun? = null
        private set

    /** False while a background sort holds the folders. */
    fun start(context: Context, request: OrganizeRequest): Boolean {
        current = OrganizeRun.startIfIdle(context, request) ?: return false
        return true
    }
}

class OrganizeRun private constructor(private val context: Context, private val request: OrganizeRequest) {
    val progress = RunProgress()
    private val cancelled = AtomicBoolean(false)

    fun cancel() = cancelled.set(true)

    private fun start(onEnd: () -> Unit) {
        thread(name = "organize") {
            try {
                notifyGallery(organize(request, DeviceMetadataReader, progress, cancelled::get))
            } catch (error: Exception) {
                progress.log("Stopped: $error")
                progress.finish(RunEnd.FAILED)
            } finally {
                busy.set(false)
                onEnd()
            }
        }
    }

    /** The gallery reads its own index, so both the old and the new paths need a rescan. */
    private fun notifyGallery(moved: List<MovedFile>) {
        if (moved.isEmpty()) return
        val paths = moved.flatMap { listOf(it.from.toString(), it.to.toString()) }
        MediaScannerConnection.scanFile(context, paths.toTypedArray(), null, null)
    }

    companion object {
        // Two runs over one gallery folder would race to move the same files.
        private val busy = AtomicBoolean(false)

        /** Starts a run on its own thread, or returns null while another run goes. */
        fun startIfIdle(context: Context, request: OrganizeRequest, onEnd: () -> Unit = {}): OrganizeRun? {
            if (!busy.compareAndSet(false, true)) return null
            return OrganizeRun(context.applicationContext, request).also { it.start(onEnd) }
        }
    }
}
