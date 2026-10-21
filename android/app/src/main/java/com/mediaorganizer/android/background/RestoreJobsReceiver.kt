package com.mediaorganizer.android.background

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Android drops gallery triggers on reboot, so the jobs go back in once the phone starts or the app updates. */
class RestoreJobsReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED || intent.action == Intent.ACTION_MY_PACKAGE_REPLACED) {
            BackgroundSorting.scheduleMissing(context)
        }
    }
}
