# tetris-android

A Tetris implementation in native Rust/NDK, deliberately built on the same
lower-level stack Bevy uses on Android — no winit, no game engine — to hit
Bevy's Android pain points firsthand (surface lifecycle, GameActivity, IME,
rotation) and produce a reproducible test bed for contributing fixes upstream.

See [design/](design/README.md) for the research and rationale behind this project
(also kept canonically in [developer-kb: bevy-android](https://github.com/sqrt57/developer-kb/blob/main/ideas/bevy-android.md)).

## Stack

| Layer | Choice |
|---|---|
| Language | Rust |
| Windowing / lifecycle | [`android-activity`](https://github.com/rust-mobile/android-activity) (GameActivity backend) — the same crate Bevy uses |
| Rendering | `wgpu` (Vulkan backend) — the same renderer Bevy uses |
| Android shell | Jetpack `GameActivity`, thin Kotlin stub (~10 lines) |
| Build | Cargo + `cargo-ndk` + Gradle |

`minSdk = 31` (Android 12, GameActivity/Vulkan baseline), `targetSdk = 36`,
`compileSdk = 36`.

## Project layout

```
src/
  game.rs        Pure Tetris logic — board, pieces, gravity, line clear. No
                 Android/GPU deps, fully unit-testable on desktop.
  lib.rs         android_main entry point, event loop.
  renderer.rs    wgpu surface bound to the current ANativeWindow.
  lifecycle.rs   Owns the renderer across surface-destroyed/recreated events.
  input.rs       Touch gesture classification (zone tap / swipe) into game actions.
app/             Gradle module: manifest, GameActivity Kotlin stub, resources.
```

## Build

```bash
# Rust side: cross-compiles for device (arm64) and emulator (x86_64), drops
# the .so into app/src/main/jniLibs. Runs automatically before every Gradle
# build via the cargoBuild task in app/build.gradle.kts.
cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build

# Full APK
./gradlew assembleDebug
```

`game.rs` has no Android dependencies and runs under a normal host `cargo test`.

## Status

Working through the build-steps table from the original research spec:

- [x] Step 1 — wgpu surface, clears to a solid color
- [x] Step 2 — suspend/resume lifecycle (renderer torn down on `TerminateWindow`, rebuilt on `InitWindow`)
- [x] Step 3 — fixed-timestep game loop drives `game::Game::tick()`
- [x] Step 4 — touch input: zone tap (left/right/rotate), swipe down (soft/hard drop)
- [x] Step 5 — Tetris game logic (pure Rust, unit-tested)
- [x] Step 6 — render the board via wgpu (instanced quads: locked cells + falling piece, per-`Kind` color)
- [x] Step 7 — soft keyboard / IME (high-score name prompt on game over, via `android-activity`'s `TextEvent`/`TextAction` bridge)
- [x] Step 8 — screen rotation / config changes (orientation unlocked; `WindowResized`/`ConfigChanged` reconfigure the wgpu surface in place, no Activity recreation)

`./gradlew assembleDebug` succeeds end to end (Rust cross-compile for arm64-v8a
and x86_64 via the `cargoBuild` task, then the Gradle/Kotlin/manifest side).
Verified running on a physical device (touch input, board rendering,
soft-keyboard name entry on game over, portrait/landscape rotation); no AVD
is set up.

All 8 build steps from the original spec are done. The board is not yet
resized/rebalanced for landscape beyond simple re-centering (it stays
portrait-shaped with wide side margins), and there's still no font/glyph
rendering — the high-score name is captured but not drawn as text.

## Toolchain

- Rust targets: `rustup target add aarch64-linux-android x86_64-linux-android`
- `cargo install cargo-ndk`
- Android NDK (installed via Android Studio SDK Manager) and `ANDROID_HOME` set
