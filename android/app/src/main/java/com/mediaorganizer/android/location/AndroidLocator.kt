package com.mediaorganizer.android.location

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.location.Location
import android.location.LocationManager
import android.os.CancellationSignal
import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.places.DeviceLocator
import java.util.TimeZone
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

private const val FIX_WAIT_SECONDS = 10L

/** Asks Android where the phone is. Call off the main thread: a fix can take up to 10 seconds. */
class AndroidLocator(private val context: Context) : DeviceLocator {
    override fun currentPoint(): Coordinates? {
        if (!hasLocationAccess(context)) return null
        val manager = context.getSystemService(LocationManager::class.java)
        val provider = listOf(LocationManager.FUSED_PROVIDER, LocationManager.NETWORK_PROVIDER, LocationManager.GPS_PROVIDER)
            .firstOrNull { it in manager.allProviders && manager.isProviderEnabled(it) }
            ?: return null
        return fix(manager, provider)?.let { Coordinates(it.latitude, it.longitude) }
    }

    override fun timeZone(): String? = TimeZone.getDefault().id

    // `001` is the world: a zone such as `UTC` has no country.
    override fun countryOfZone(zone: String): String? = try {
        android.icu.util.TimeZone.getRegion(zone).takeIf { it != "001" }
    } catch (_: IllegalArgumentException) {
        null
    }

    @Suppress("MissingPermission")
    private fun fix(manager: LocationManager, provider: String): Location? {
        val done = CountDownLatch(1)
        var found: Location? = null
        val cancel = CancellationSignal()
        val executor = Executors.newSingleThreadExecutor()
        try {
            manager.getCurrentLocation(provider, cancel, executor) { location ->
                found = location
                done.countDown()
            }
            if (!done.await(FIX_WAIT_SECONDS, TimeUnit.SECONDS)) cancel.cancel()
        } finally {
            executor.shutdown()
        }
        return found ?: manager.getLastKnownLocation(provider)
    }
}

fun hasLocationAccess(context: Context): Boolean =
    context.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) == PackageManager.PERMISSION_GRANTED
