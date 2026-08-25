package com.tonic.songbook

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

/**
 * Immersive sticky system bars, plus a deep link to open All-files settings
 * (`tonic://request-all-files`) without JNI from Rust.
 */
class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    window.decorView.post { hideSystemBars() }
    maybeOpenAllFilesSettings(intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    setIntent(intent)
    maybeOpenAllFilesSettings(intent)
  }

  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus) {
      window.decorView.post { hideSystemBars() }
    }
  }

  private fun maybeOpenAllFilesSettings(intent: Intent?) {
    val data = intent?.data ?: return
    if (data.scheme != "tonic" || data.host != "request-all-files") {
      return
    }
    openAllFilesSettings()
    // Prevent re-opening on every recreate.
    setIntent(Intent(intent).setData(null))
  }

  private fun openAllFilesSettings() {
    runCatching {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        try {
          startActivity(
            Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION).apply {
              data = Uri.parse("package:$packageName")
            },
          )
        } catch (_: Exception) {
          startActivity(Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION))
        }
      } else {
        startActivity(
          Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
            data = Uri.parse("package:$packageName")
          },
        )
      }
    }
  }

  private fun hideSystemBars() {
    runCatching {
      WindowCompat.setDecorFitsSystemWindows(window, false)
      val controller = WindowCompat.getInsetsController(window, window.decorView)
      controller.systemBarsBehavior =
        WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
      controller.hide(WindowInsetsCompat.Type.systemBars())
    }
  }
}
