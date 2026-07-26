package com.sqrt57.tetris

import android.os.Bundle
import com.google.androidgamesdk.GameActivity

// Thin shell. All game logic, rendering, and lifecycle handling live in the
// Rust cdylib (android_main in src/lib.rs), loaded via the android.app.lib_name
// meta-data entry in AndroidManifest.xml. This class exists only because
// AAB/Play Store packaging and the Activity lifecycle require a JVM entry point.
class MainActivity : GameActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }
}
