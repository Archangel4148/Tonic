package com.tonic.songbook

import android.os.Bundle
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

/**
 * Immersive sticky: hide status + nav bars; swipe from edge to peek.
 * Kept minimal — heavy window flags here have crashed some WebView/Wry setups.
 */
class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    window.decorView.post { hideSystemBars() }
  }

  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus) {
      window.decorView.post { hideSystemBars() }
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
