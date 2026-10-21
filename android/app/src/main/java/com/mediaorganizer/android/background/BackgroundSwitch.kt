package com.mediaorganizer.android.background

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.PowerManager
import android.provider.Settings
import android.widget.CompoundButton
import com.mediaorganizer.android.hasAllFilesAccessOrAsk

/** The "Sort new photos in the background" switch. [saveFolders] stores the fields the jobs will read. */
class BackgroundSwitch(private val activity: Activity, switch: CompoundButton, private val saveFolders: () -> Unit) {
    init {
        switch.isChecked = BackgroundSorting.isEnabled(activity)
        switch.setOnCheckedChangeListener { _, checked -> onToggled(switch, checked) }
        BackgroundSorting.scheduleMissing(activity)
    }

    private fun onToggled(switch: CompoundButton, checked: Boolean) {
        if (checked && !activity.hasAllFilesAccessOrAsk()) {
            switch.isChecked = false
            return
        }
        saveFolders()
        BackgroundSorting.setEnabled(activity, checked)
        if (checked) askToSkipBatteryLimits()
    }

    /** Android holds back jobs of a rarely opened app for up to a day. An app without battery limits runs on time. */
    private fun askToSkipBatteryLimits() {
        val power = activity.getSystemService(PowerManager::class.java)
        if (power.isIgnoringBatteryOptimizations(activity.packageName)) return
        val uri = Uri.parse("package:${activity.packageName}")
        activity.startActivity(Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, uri))
    }
}
