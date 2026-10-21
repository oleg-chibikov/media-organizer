package com.mediaorganizer.android

import android.app.Activity
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.WindowManager
import android.widget.Toast
import com.mediaorganizer.android.background.BackgroundSwitch
import com.mediaorganizer.android.folders.FolderFields
import com.mediaorganizer.android.organize.RequestCheck
import com.mediaorganizer.android.organize.checkRequest

private const val REFRESH_MILLIS = 200L

class MainActivity : Activity() {
    private lateinit var screen: RunScreen
    private lateinit var folders: FolderFields
    private val handler = Handler(Looper.getMainLooper())
    private val refresh = object : Runnable {
        override fun run() {
            val snapshot = ActiveRun.current?.progress?.snapshot() ?: return
            screen.show(snapshot)
            if (snapshot.finished) showRunning(false) else handler.postDelayed(this, REFRESH_MILLIS)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        screen = RunScreen(this)
        folders = FolderFields(this, screen.source, screen.destination, screen.pickSource, screen.pickDestination)
        BackgroundSwitch(this, screen.background, folders::save)
        screen.start.setOnClickListener { onStartPressed() }
    }

    override fun onResume() {
        super.onResume()
        val running = ActiveRun.current?.progress?.snapshot()?.finished == false
        showRunning(running)
        handler.post(refresh)
    }

    override fun onPause() {
        super.onPause()
        handler.removeCallbacks(refresh)
        folders.save()
    }

    private fun onStartPressed() {
        val active = ActiveRun.current
        if (active != null && !active.progress.snapshot().finished) {
            active.cancel()
            return
        }
        if (!hasAllFilesAccessOrAsk()) return
        folders.save()
        when (val check = checkRequest(folders.source, folders.destination)) {
            is RequestCheck.Invalid -> Toast.makeText(this, check.message, Toast.LENGTH_LONG).show()
            is RequestCheck.Valid -> {
                if (!ActiveRun.start(this, check.request)) {
                    Toast.makeText(this, R.string.background_busy, Toast.LENGTH_LONG).show()
                    return
                }
                showRunning(true)
                handler.removeCallbacks(refresh)
                handler.post(refresh)
            }
        }
    }

    /** Keeps the screen on while sorting: Android freezes a backgrounded app and the run pauses. */
    private fun showRunning(running: Boolean) {
        screen.setRunning(running)
        if (running) {
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        } else {
            window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }
}
