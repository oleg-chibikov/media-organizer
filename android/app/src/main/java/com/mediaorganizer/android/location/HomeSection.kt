package com.mediaorganizer.android.location

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import com.mediaorganizer.android.R
import com.mediaorganizer.android.places.HomeLocation
import com.mediaorganizer.android.places.HomeSource
import com.mediaorganizer.android.places.findHomeFromDevice
import com.mediaorganizer.android.places.locatedHome
import kotlin.concurrent.thread

private const val START_REQUEST = 21
private const val LOCATE_REQUEST = 22
private const val ASKED_ON_START = "asked_on_start"

/** The Home row: the saved home and where it came from, with Change and Clear, as the desktop Home section. */
class HomeSection(
    private val activity: Activity,
    private val status: TextView,
    change: Button,
    private val clear: Button,
) {
    private val saved = SavedHome(activity)
    private val prompts = activity.getSharedPreferences("home_prompt", Context.MODE_PRIVATE)

    init {
        change.setOnClickListener { HomeDialog(activity, ::replace, ::locate).show() }
        clear.setOnClickListener { replace(null) }
        show()
    }

    /** Shows the saved home. A run can learn one, so the screen calls this after each run. */
    fun show() {
        val home = saved.saved()
        status.text = if (home == null) activity.getString(R.string.home_not_set) else statusText(home)
        clear.isEnabled = home != null
    }

    /** With no home or only a time zone guess, finds home from the phone, as desktop does on start. Asks for location once. */
    fun findOnStart() {
        val home = saved.saved()
        if (home != null && home.source != HomeSource.TIME_ZONE) return
        if (!hasLocationAccess(activity) && !prompts.getBoolean(ASKED_ON_START, false)) {
            prompts.edit().putBoolean(ASKED_ON_START, true).apply()
            activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_COARSE_LOCATION), START_REQUEST)
            return
        }
        inBackground({ findHomeFromDevice(saved, AndroidLocator(activity), AppGeocoder.of(activity)) }) { show() }
    }

    fun onPermissionResult(requestCode: Int, grants: IntArray) {
        val granted = grants.firstOrNull() == PackageManager.PERMISSION_GRANTED
        when (requestCode) {
            // Without access the time zone still names a home.
            START_REQUEST -> findOnStart()
            LOCATE_REQUEST -> if (granted) locate() else noFix()
        }
    }

    private fun locate() {
        if (!hasLocationAccess(activity)) {
            activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_COARSE_LOCATION), LOCATE_REQUEST)
            return
        }
        status.setText(R.string.home_locating)
        inBackground({ locatedHome(AndroidLocator(activity), AppGeocoder.of(activity)) }) { home ->
            if (home == null) noFix() else replace(home)
            show()
        }
    }

    private fun replace(home: HomeLocation?) {
        saved.replace(home)
        show()
    }

    private fun noFix() = Toast.makeText(activity, R.string.home_no_fix, Toast.LENGTH_LONG).show()

    private fun statusText(home: HomeLocation): String {
        val source = when (home.source) {
            HomeSource.MANUAL -> R.string.home_source_manual
            HomeSource.DEVICE -> R.string.home_source_device
            HomeSource.LEARNED -> R.string.home_source_learned
            HomeSource.TIME_ZONE -> R.string.home_source_time_zone
        }
        return activity.getString(R.string.home_status, home.name, home.country, activity.getString(source))
    }

    private fun <T> inBackground(work: () -> T, done: (T) -> Unit) = thread {
        val result = work()
        activity.runOnUiThread { if (!activity.isDestroyed) done(result) }
    }
}
