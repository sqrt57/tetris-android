import org.gradle.api.tasks.Exec

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.sqrt57.tetris"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.sqrt57.tetris"
        minSdk = 31
        targetSdk = 36
        versionCode = 1
        versionName = "0.1"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    implementation("androidx.games:games-activity:3.0.4")
    implementation("androidx.appcompat:appcompat:1.7.0")
}

// Builds the Rust cdylib for both device (arm64) and emulator (x86_64) ABIs and
// drops the resulting .so files directly into jniLibs, ahead of every build.
val cargoBuild = tasks.register<Exec>("cargoBuild") {
    workingDir = rootDir
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", "app/src/main/jniLibs",
        "build",
    )
}

tasks.named("preBuild") {
    dependsOn(cargoBuild)
}
