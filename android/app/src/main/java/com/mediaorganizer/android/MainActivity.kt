package com.mediaorganizer.android

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.view.WindowManager
import android.widget.Toast
import com.mediaorganizer.android.organize.RequestCheck
import com.mediaorganizer.android.organize.checkRequest

private const val REFRESH_MILLIS = 200L

class MainActivity : Activity() {
    private lateinit var screen: RunScreen
    private lateinit var folders: SavedFolders
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
        folders = SavedFolders(this)
        screen.source.setText(folders.source)
        screen.destination.setText(folders.destination)
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
        folders.save(screen.source.text.toString(), screen.destination.text.toString())
    }

    private fun onStartPressed() {
        val active = ActiveRun.current
        if (active != null && !active.progress.snapshot().finished) {
            active.cancel()
            return
        }
        if (!Environment.isExternalStorageManager()) {
            Toast.makeText(this, R.string.allow_all_files, Toast.LENGTH_LONG).show()
            startActivity(Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:$packageName")))
            return
        }
        when (val check = checkRequest(screen.source.text.toString(), screen.destination.text.toString())) {
            is RequestCheck.Invalid -> Toast.makeText(this, check.message, Toast.LENGTH_LONG).show()
            is RequestCheck.Valid -> {
                ActiveRun.start(this, check.request)
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
