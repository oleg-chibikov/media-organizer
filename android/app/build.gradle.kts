plugins {
    id("com.android.application")
}

android {
    namespace = "com.mediaorganizer.android"
    compileSdk = 35
    // Matches the build-tools that build-apk installs, so Gradle downloads no other version.
    buildToolsVersion = "36.0.0"

    defaultConfig {
        applicationId = "com.mediaorganizer.android"
        minSdk = 30
        targetSdk = 35
        versionCode = System.getenv("ANDROID_VERSION_CODE")?.toInt() ?: 1
        versionName = System.getenv("ANDROID_VERSION_NAME") ?: "dev"
    }

    // GitHub releases sign with one key so a new APK installs over the old one.
    // Local builds fall back to the debug key.
    val releaseKeystore = System.getenv("ANDROID_KEYSTORE_FILE")?.let(::file)
    signingConfigs {
        if (releaseKeystore != null) {
            create("release") {
                storeFile = releaseKeystore
                storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
                keyAlias = "media-organizer"
                keyPassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName(if (releaseKeystore != null) "release" else "debug")
        }
    }

    buildFeatures {
        // MainActivity puts BuildConfig.VERSION_NAME in the screen title.
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    // The desktop app's place tables, so both apps name places alike. NOTICE.txt credits their sources.
    sourceSets.getByName("main").assets.srcDir("../../src-tauri/assets")
}

dependencies {
    testImplementation("junit:junit:4.13.2")
}
