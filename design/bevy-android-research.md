# Bevy, Android Game Development & Project Research
> Summary of research session — June 2026
>
> Copied from [developer-kb/ideas/bevy-android.md](https://github.com/sqrt57/developer-kb/blob/main/ideas/bevy-android.md)
> on 2026-07-26. That copy is canonical; this one is a snapshot for repo-local context.

---

## Table of Contents

1. [Rust Game Engines Overview](#rust-game-engines)
2. [Bevy Deep Dive](#bevy)
3. [Code-Focused Game Frameworks (Cross-Language)](#cross-language-frameworks)
4. [Mobile-Focused Frameworks](#mobile-frameworks)
5. [LibGDX Deep Dive](#libgdx)
6. [ECS Libraries for JVM](#ecs-jvm)
7. [Java vs Kotlin for Claude Code](#java-vs-kotlin)
8. [Framework Comparison Matrix](#comparison-matrix)
9. [Android NDK & Lifecycle](#android-ndk)
10. [Android API Level Strategy](#android-api-levels)
11. [wgpu Explained](#wgpu)
12. [Tetris Project Spec](#tetris-project-spec)
13. [Contributing to Bevy Android](#contributing-to-bevy)

---

## 1. Rust Game Engines Overview {#rust-game-engines}

The four main Rust game engines as of early 2026:

| Engine | Version | Focus | Architecture |
|---|---|---|---|
| **Bevy** | 0.18 | 2D & 3D | ECS, code-only |
| **Macroquad** | 0.4.14 | 2D | Minimal, Raylib-inspired |
| **ggez** | 0.9.3 | 2D | Comfortable defaults, mature |
| **Fyrox** | 0.36.2 | 3D | Has visual scene editor |

**Bevy** is the standout — 44,000+ GitHub stars as of January 2026, hot reloading stable, indie titles shipped (including *Toroban* on Steam). Its sweet spot is simulation-heavy games (factory sims, colony sims, roguelikes) where ECS performance pays off. Solid WebGPU story for web deployment.

---

## 2. Bevy Deep Dive {#bevy}

### Platform Support

| Platform | Status |
|---|---|
| Desktop (Windows / macOS / Linux) | Production-ready, works out of the box |
| Web (WASM) | Good — WebGPU rendering, game jam target |
| Android | Possible, rough, needs contributors |
| iOS | Possible, rough, needs contributors |
| Consoles | DIY — no official path, no middleware partnerships |

Community members have gotten Bevy running on the Game Boy Advance and Playdate via ongoing `no_std` work.

### Current Version State

- **Stable:** Bevy 0.18 (January 2026) — GPU-driven rendering, ~3x perf improvement on complex 3D scenes over 0.15
- **RC:** Bevy 0.19-rc.2 (May 22, 2026) — focused on ECS performance, rendering correctness, UI fixes; **not** mobile

### Android Status (Known Issues)

| Problem | Details |
|---|---|
| Activity type | Still uses `NativeActivity` (old) — should be `GameActivity` |
| `minSdkVersion` | Inconsistent across examples (~16–23) |
| `targetSdkVersion` | 33 — outdated, Play Store requires 35 |
| Output format | APK only via `cargo-apk` — Play Store requires AAB, which `cargo-apk` cannot produce |
| Crash on backgrounding | Surface destruction not handled correctly |
| IME / soft keyboard | Does not work |
| Config changes (rotation) | Crashes in Bevy apps |
| Mobile contribution | Gated on community adoption, not planned engine work |

There is a community project (`bevy-in-app`) for integrating Bevy into an existing Android app via `SurfaceView` rather than as a full-screen app.

---

## 3. Code-Focused Game Frameworks (Cross-Language) {#cross-language-frameworks}

### Rust
- **Bevy** — ECS, all-code, best Claude Code fit
- **Macroquad** — canvas + drawing functions, no architecture imposed
- **ggez** — 2D, mature, good defaults

### JavaScript / TypeScript
- **Phaser 4** — benchmark for code-first JS/TS 2D game dev, huge community

### C# / .NET
- **MonoGame** — no built-in editor by design, complete control, XNA heritage, great for 2D

### Low-Level / Language-Agnostic
- **SDL** — windowing + input, "library not framework" philosophy
- **Raylib** — simple C library, bindings for almost every language including Rust (`raylib-rs`)
- **sokol** — minimal, single-file C headers

Modern languages (Zig, Odin, Rust) have made low-level game programming far less painful than C/C++, fueling a renaissance in engine-free development.

---

## 4. Mobile-Focused Code-First Frameworks {#mobile-frameworks}

| Framework | Language | Android | iOS | Notes |
|---|---|---|---|---|
| **LibGDX** | Java / Kotlin | ✅ Production-ready | ✅ | Battle-tested, batteries-included |
| **LÖVE2D** | Lua | ✅ (setup needed) | ✅ | Very lightweight |
| **Raylib / SDL** | C (+ bindings) | ✅ (NDK manual) | ✅ (Xcode) | Low-level, max control |
| **Phaser** | JS / TS | ✅ (browser) | ✅ (browser) | No app store without wrapper |
| **Godot 4.6** | GDScript / C# | ✅ | ✅ | MIT, zero royalties; Jan 2026: AAB, StoreKit 2, Google Play Billing |
| **Bevy** | Rust | ⚠️ Rough | ⚠️ Rough | See above |

For production mobile today: **Godot** or **LibGDX** are the pragmatic choices.

---

## 5. LibGDX Deep Dive {#libgdx}

### What it is
Free and open-source Java game framework (Apache 2.0), C/C++ for performance-critical code. Single codebase targets Windows, Linux, macOS, Android, iOS, and Web via OpenGL ES.

Completely code-centric — no editor. Does not force any design pattern.

### Batteries Included
- Audio: WAV, MP3, OGG playback and streaming
- Input: mouse, keyboard, touchscreen, controllers, accelerometer, gyroscope, gesture detection
- Math: vectors, matrices, quaternions (C-accelerated via JNI)
- 2D Physics: Box2D (JNI wrapper)
- 3D Physics: Bullet
- AI: pathfinding, behaviour trees, FSMs via `gdx-ai`

### Current State (2026)
- Latest release: **1.14.1** (May 2026) — bug fixes including Android ANR/crash fix
- 36th LibGDX Jam ran March 2026 — active community
- Setup tool `gdx-liftoff` now supports Java 25 and 26

### Android Story
Mature, solved, production-shipped. The NDK integration is battle-tested. The Box2D JNI bindings and Android backend are worth studying as reference for how Bevy should work.

---

## 6. ECS Libraries for JVM {#ecs-jvm}

| Library | Language | Performance | Notes |
|---|---|---|---|
| **Ashley** | Java | Slowest | Official libGDX org, easiest integration |
| **Artemis-ODB** | Java | Fast | Bytecode weaving, more opinionated |
| **Fleks** | Kotlin (KMP) | ~1.2x Artemis | Modern, idiomatic, no reflection, closest to Bevy |

### Fleks Details
- Current version: **2.14**
- Multiplatform: JVM, Kotlin/Native, Wasm (Android, iOS, Desktop, Web)
- Version 2.x combined KMP and JVM flavors into one, removing all reflection
- Used in KorGE via `korge-fleks` integration
- Benchmark: Ashley is slowest by far; Fleks ≈ 1.2x faster than Artemis on AddRemove

**Fleks is the closest Kotlin equivalent to Bevy's ECS** — type-safe queries, systems-first mindset, KMP support.

---

## 7. Java vs Kotlin for Claude Code {#java-vs-kotlin}

### General State (2026)
- Kotlin 2.3.0 (December 2025): K2 compiler builds large projects 40–50% faster than Java
- 80%+ of new Android modules at Google are Kotlin
- Kotlin produces ~40% less code than Java, ~33% fewer crashes (compile-time null safety)
- Java: TIOBE 17.45% vs Kotlin 1.82% — Java wins on legacy and job market

### Interoperability
Kotlin is 100% interoperable with Java. Any Java library works in a Kotlin project without special configuration. This completely neutralizes Java's ecosystem advantage.

### For Claude Code Specifically — Kotlin Wins Clearly

| Factor | Why it matters for Claude Code |
|---|---|
| Less boilerplate | Smaller context window consumption per file |
| Coroutines | Linear-looking async code, easier to reason about across edits |
| Compile-time null safety | Prevents ~80% of NPEs — shorter edit/compile/fix loops |
| Kotlin DSLs | Read like structured English (Fleks, Gradle KTS, Ktor) |

**Verdict:** For greenfield LibGDX projects in 2026, Kotlin is the default choice — gap is even wider for Claude Code than for human developers.

---

## 8. Framework Comparison Matrix {#comparison-matrix}

| | **Bevy** | **Flutter** | **LibGDX + Fleks** |
|---|---|---|---|
| Layer | Full engine | Full framework | Rendering lib + ECS lib |
| Language | Rust | Dart (C++ engine) | Kotlin |
| Android shell | NDK / NativeActivity (old) | Java/Kotlin SDK (modern) | Java/Kotlin SDK (modern) |
| Graphics API | wgpu → Vulkan | Impeller → Vulkan | OpenGL ES |
| ECS | Built-in (archetype) | None | Fleks (archetype, KMP) |
| `minSdkVersion` | ~16–23 (inconsistent) | 24 (enforced) | Your choice (24+ typical) |
| `targetSdkVersion` | 33 (outdated) | 35 (current) | Your choice |
| Output format | APK only | APK + AAB | APK + AAB |
| Android maturity | Rough | Production-ready | Production-ready |
| Multiplatform | Desktop+Web good, Mobile rough | All platforms | Android+iOS+Desktop+Web |
| Code-first | Yes | Yes | Yes |

### Flutter's Android Architecture (Reference)
Flutter uses two layers:
1. **Java/Kotlin SDK** — `FlutterActivity` lifecycle, platform channels, plugins, input, accessibility
2. **NDK / C++** — Flutter engine itself (Dart VM, Impeller renderer) — developer-invisible

Flutter is ~2–3 years ahead of Bevy on Android maturity. Centralized `flutter.gradle` version management is worth studying as a model for Bevy's ecosystem.

---

## 9. Android NDK & Lifecycle {#android-ndk}

### Google's Two Official Approaches

**Approach 1: JNI Bridge** (most apps)
Write app in Java/Kotlin, call NDK via JNI for performance-critical code. Flutter model.

**Approach 2: NativeActivity / GameActivity** (games)
Implement lifecycle callbacks entirely in native code. Android SDK provides the activity class, native code owns everything else.

### NativeActivity vs GameActivity

| | NativeActivity | GameActivity |
|---|---|---|
| Status | Legacy | **Recommended (replaces NativeActivity)** |
| Part of | Android framework (yearly releases) | Jetpack library (biweekly releases) |
| Rendering surface | ANativeWindow | **SurfaceView** (easier UI integration) |
| Input handling | InputQueue | `android_input_buffer` (new, better) |
| IME / soft keyboard | Essentially none | **GameTextInput integration** |
| Base class | Activity | **AppCompatActivity** (full Jetpack access) |
| Backwards compatible | — | API level 19+ |
| Bevy current state | Uses this ❌ | Should migrate here |

### The `native_app_glue` Threading Model

Both NativeActivity and GameActivity work with `native_app_glue`:
- Runs in its own **native thread** (separate from Android's main thread)
- Game code polls events queued inside `native_app_glue`
- In Rust: the `android-activity` crate wraps this — what Bevy uses

### Critical Lifecycle Facts

> Your android-activity entrypoints are tied to the lifecycle of your **Activity**, not your **application process**. If the Activity is destroyed and re-created, a new native entrypoint instance is invoked.

The lifecycle native code **must** handle:

```
onCreate → onStart → onResume → [RUNNING]
                                    ↓
                              onPause → onStop → onDestroy
                                    ↓
                    Surface DESTROYED → must drop all GPU resources
                                    ↓
                    Surface RECREATED → must reinitialize renderer
```

Using NativeActivity or GameActivity does NOT exempt you from lifecycle handling. You must handle all cases Java apps handle.

### What Requires the Java/Kotlin SDK Layer (Cannot be Pure NDK)

| Feature | Why |
|---|---|
| IME / soft keyboard | Requires `InputMethodManager` (Java SDK) |
| Runtime permissions | `Activity.requestPermissions()` — Java only |
| Audio focus | `AudioManager` — SDK only |
| Notifications, Bluetooth, in-app billing | SDK only |
| Foreground services, background work | SDK only |
| AAB packaging for Play Store | Gradle — NDK tools cannot produce AAB |

**Bottom line:** Pure NDK works for rendering + touch input only. A thin Kotlin stub + GameActivity is the correct modern architecture. The Kotlin layer is ~10 lines.

---

## 10. Android API Level Strategy {#android-api-levels}

### Current Device Coverage (May 28, 2026)

| Min API | Android version | Cumulative coverage |
|---|---|---|
| API 36 | Android 16 | 22.3% |
| API 35 | Android 15 | 41.0% |
| API 34 | Android 14 | 54.5% |
| API 33 | Android 13 | 68.9% |
| **API 31** | **Android 12** | **78.8%** |
| API 30 | Android 11 | 86.9% |
| API 29 | Android 10 | 91.1% |
| API 28 | Android 9 | 93.5% |
| API 26 | Android 8.0 | 96.1% |

### Play Store Requirements
- `targetSdkVersion` must be **35+** for new apps and updates (since August 31, 2025)

### Recommendation for Project

```
minSdkVersion   = 31   (Android 12) — GameActivity requires this, Vulkan mandatory, 78.8% coverage
targetSdkVersion = 35   (Android 15) — Play Store requirement
compileSdkVersion = 35
```

### Where Bevy Stands vs Target

| | Bevy today | Should be |
|---|---|---|
| `minSdkVersion` | ~16–23 (inconsistent) | 31 |
| `targetSdkVersion` | 33 | 35 |
| Activity type | NativeActivity | GameActivity |
| Output format | APK (cargo-apk) | AAB (Gradle) |

---

## 11. wgpu Explained {#wgpu}

wgpu is a Rust implementation of the WebGPU API standard that runs everywhere, not just the web.

### Backend Translation

| Platform | Native API used |
|---|---|
| Windows | Vulkan or DirectX 12 |
| macOS / iOS | Metal |
| Linux | Vulkan |
| **Android** | **Vulkan (primary) / OpenGL ES (fallback)** |
| Web | WebGPU / WebGL2 fallback |

Write rendering code once against wgpu — runs on all platforms. This is why Bevy uses it.

### Core Concepts (Backend Developer Mental Model)

Like a database connection pool, but for GPU work. Everything is explicit:

- **Adapters** — which GPU to use
- **Devices** — your connection to that GPU
- **Buffers** — GPU memory (vertices, uniforms)
- **Textures** — images on GPU
- **Pipelines** — compiled shader programs + fixed-function state
- **Command encoders** — record draw calls, submit in batches

Nothing happens implicitly. Explicit model = faster than OpenGL (driver doesn't guess intent).

### Shader Language: WGSL
WebGPU Shading Language — Rust-ish, compiles to SPIR-V / MSL / HLSL as needed. A simple Tetris colored-quad shader is ~20 lines.

### Why wgpu for the Tetris Project
1. **It's what Bevy uses** — understanding wgpu surface lifecycle on Android = understanding Bevy's Android renderer
2. **Surface lifecycle is explicit** — surface destruction on backgrounding is painful and visible, which is the pain point to learn
3. **Same code runs on desktop** — develop and debug without a device for most work

---

## 12. Tetris Project Spec {#tetris-project-spec}

> **Started:** promoted to [github.com/sqrt57/tetris-android](https://github.com/sqrt57/tetris-android)
> (2026-07-26). The spec below is the original research; the repo's README tracks
> current status against the build-steps table.

> **Goal:** Build a Tetris game in native Rust/NDK that deliberately exercises the exact pain points Bevy faces on Android, producing transferable knowledge for contributing to Bevy's Android port.

### Platform & Toolchain

| Item | Choice |
|---|---|
| Target API | `minSdkVersion = 31`, `targetSdkVersion = 35` |
| Rust targets | `aarch64-linux-android` (device), `x86_64-linux-android` (emulator) |
| NDK version | r27 (stable 2026) |
| Build system | Cargo + `cargo-ndk` + Gradle (APK/AAB shell) |

### Core Libraries

| Library | Purpose |
|---|---|
| `android-activity` (GameActivity backend) | Android lifecycle abstraction — same layer Bevy uses |
| `ndk` crate | Safe Rust bindings to NDK types (ANativeWindow, ALooper) |
| `wgpu` (Vulkan backend) | Rendering — same as Bevy |
| `oboe` crate | Audio (optional first pass) |

**No winit** — handle `ANativeWindow` directly to learn what winit does for Bevy.

### Project Structure

```
tetris-android/
├── Cargo.toml
├── build.gradle
├── app/
│   ├── src/main/
│   │   ├── AndroidManifest.xml    # GameActivity declaration
│   │   └── res/                   # icons, strings
│   └── build.gradle
└── src/
    ├── lib.rs          # android_main entry point
    ├── game.rs         # Tetris logic (pure Rust, zero Android deps)
    ├── renderer.rs     # wgpu surface + draw calls
    ├── input.rs        # touch/key event handling
    └── lifecycle.rs    # suspend/resume/surface lost handling
```

Separating `game.rs` enables full unit testing on desktop.

### Key Config

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]   # Android loads a .so, not a binary

[dependencies]
android-activity = { version = "0.6", features = ["game-activity"] }
ndk = "0.9"
wgpu = { version = "0.20", features = ["vulkan"] }
```

```bash
# Build command
cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build --release
```

### Build Steps

| Step | What you build | Bevy pain point exercised |
|---|---|---|
| 1 | wgpu surface rendering a solid color | NDK setup, GameActivity init, ANativeWindow → wgpu surface |
| 2 | Suspend/resume lifecycle | Surface destruction/recreation — Bevy crashes on backgrounding |
| 3 | Fixed-timestep game loop via ALooper | Bevy's schedule vs Android event loop |
| 4 | Touch input (left/right/rotate/drop) | GameActivity input pipeline vs Bevy's abstraction gaps |
| 5 | Tetris game logic (pure Rust) | N/A — portable, testable |
| 6 | Render board via wgpu | Surface format negotiation — color space issues on Android |
| 7 | Soft keyboard / IME | GameActivity IME — doesn't work in Bevy at all |
| 8 | Screen rotation / config changes | Rotation crashes in Bevy apps |

### Kotlin Stub (the only Java/Kotlin code needed)

```kotlin
class MainActivity : GameActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }
}
```

---

## 13. Contributing to Bevy Android {#contributing-to-bevy}

### Entry Points
- **GitHub:** `bevyengine/bevy` — search labels `platform-android`, `mobile`
- **Discord:** Bevy's active Discord, `#platform-android` channel
- **This Week in Bevy:** community progress tracker

### What Kind of Help is Most Needed
The Bevy team's position: mobile support stagnates because not enough developers are *using* Bevy on Android and hitting issues. The highest-value contributions:
1. Build something with Bevy targeting Android
2. Document what breaks
3. File issues with reproduction cases, or PRs fixing them

### Specific Known Gaps to Fix

| Gap | Work involved |
|---|---|
| Migrate NativeActivity → GameActivity | Architecture change in `winit` Android backend |
| Bump `targetSdkVersion` to 35 | Update examples + CI |
| Standardize `minSdkVersion` to 31 | Ecosystem-wide consistency |
| AAB output support | Replace `cargo-apk` with Gradle-native workflow |
| Surface lifecycle crash | Fix wgpu surface drop/recreate on Activity pause |
| IME / soft keyboard | Implement GameTextInput bridge |
| Rotation / config change crash | Handle surface recreation on config change |

### The `android-activity` Crate is the Key Layer
Most Bevy Android pain points live either in `android-activity`, in `winit`'s Android backend, or in how Bevy interacts with winit. Contributing to `android-activity` directly improves Bevy indirectly.

### Background Knowledge to Acquire First
1. **NDK directly** — build a small native Android app (C or Rust) to understand lifecycle hooks
2. **`android-activity` crate** — study its API, it's the Rust abstraction Bevy uses
3. **`winit` Android backend** — Bevy's windowing goes through winit, which goes through `android-activity`
4. **GameActivity migration guide** — `developer.android.com/games/agdk/game-activity/migrate-native-activity`
5. **The Tetris project** — builds all of the above from scratch

### Your Background as an Asset
Platform/backend experience maps well to the lower-level integration work needed: window lifecycle, input handling, build tooling. This is exactly where the Android port needs depth — not gameplay-level fixes.

---

*Research session: June 2026. Key sources: Bevy GitHub, Android Developers documentation, crates.io, apilevels.com.*
