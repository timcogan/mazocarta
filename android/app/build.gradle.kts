plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

val repoRoot = rootProject.projectDir.parentFile
val webDir = repoRoot.resolve("web")
val cargoVersion = Regex("""(?m)^version\s*=\s*"([^"]+)"""")
    .find(repoRoot.resolve("Cargo.toml").readText())
    ?.groupValues
    ?.get(1)
    ?: error("Could not read package version from Cargo.toml")
val buildWebAssets by tasks.registering(Exec::class) {
    group = "build"
    description = "Build the WASM web bundle and generated PWA icons."
    workingDir = repoRoot
    commandLine(repoRoot.resolve("scripts/build-web.sh").path)

    inputs.dir(repoRoot.resolve("src"))
    inputs.files(
        repoRoot.resolve("Cargo.lock"),
        repoRoot.resolve("Cargo.toml"),
        repoRoot.resolve("scripts/build-web.sh"),
        repoRoot.resolve("scripts/render-combat-icons.sh"),
        repoRoot.resolve("scripts/render-pwa-icons.sh"),
        repoRoot.resolve("scripts/vendor-qr-libs.mjs"),
        webDir.resolve("icons/combat/arrow.svg"),
        webDir.resolve("icons/combat/deck.svg"),
        webDir.resolve("icons/combat/energy.svg"),
        webDir.resolve("icons/combat/heart.svg"),
        webDir.resolve("icons/combat/shield.svg"),
        webDir.resolve("mazocarta.svg"),
        webDir.resolve("jsqr.js"),
        webDir.resolve("qrcode.bundle.mjs"),
    )
    outputs.files(
        webDir.resolve("mazocarta.wasm"),
        webDir.resolve("apple-touch-icon.png"),
        webDir.resolve("icons/combat/arrow.png"),
        webDir.resolve("icons/combat/deck.png"),
        webDir.resolve("icons/combat/energy.png"),
        webDir.resolve("icons/combat/heart.png"),
        webDir.resolve("icons/combat/shield.png"),
        webDir.resolve("icons/icon-192.png"),
        webDir.resolve("icons/icon-512.png"),
    )
}

val syncAndroidAssets by tasks.registering(Exec::class) {
    group = "build"
    description = "Sync web assets and generated launcher icons into the Android app."
    dependsOn(buildWebAssets)
    workingDir = repoRoot
    commandLine(repoRoot.resolve("scripts/android-sync-assets.sh").path)

    inputs.dir(webDir)
    inputs.files(
        repoRoot.resolve("scripts/android-sync-assets.sh"),
        repoRoot.resolve("scripts/render-android-icons.sh"),
    )
    outputs.dir(project.layout.projectDirectory.dir("src/main/assets/site"))
    outputs.files(
        project.layout.projectDirectory.file("src/main/res/drawable/ic_launcher_foreground.xml"),
        project.layout.projectDirectory.file("src/main/res/mipmap-anydpi-v26/ic_launcher.xml"),
        project.layout.projectDirectory.file("src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml"),
    )
}

tasks.named("preBuild") {
    dependsOn(syncAndroidAssets)
}

android {
    namespace = "com.mazocarta.android"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.mazocarta.android"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = cargoVersion

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("androidx.webkit:webkit:1.13.0")
    implementation("androidx.core:core-ktx:1.15.0")
}
