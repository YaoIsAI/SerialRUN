#!/bin/bash
# Build SerialRUN Mobile APK from scratch
# Usage: bash scripts/build_apk.sh

set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="$PROJECT_DIR/apk_build"
ANDROID_SDK="/c/Users/yao/android-sdk"
BUILD_TOOLS="$ANDROID_SDK/build-tools/34.0.0"
NDK="$ANDROID_SDK/ndk/26.1.10909125"
VERSION="0.1.0"

export PATH="$NDK/toolchains/llvm/prebuilt/windows-x86_64/bin:$PATH"

echo "=== Building Rust libraries for Android ==="
cd "$PROJECT_DIR"
cargo build --release --target aarch64-linux-android -p serialrun-mobile-app --lib 2>&1 | tail -3
cargo build --release --target x86_64-linux-android -p serialrun-mobile-app --lib 2>&1 | tail -3

echo "=== Setting up APK structure ==="
APK_TMP="$BUILD_DIR/apk_tmp"
rm -rf "$APK_TMP"
mkdir -p "$APK_TMP/lib/arm64-v8a"
mkdir -p "$APK_TMP/lib/x86_64"
mkdir -p "$APK_TMP/classes"
mkdir -p "$APK_TMP/META-INF"

# Copy .so files
cp "$PROJECT_DIR/target/aarch64-linux-android/release/libserialrun.so" "$APK_TMP/lib/arm64-v8a/"
cp "$PROJECT_DIR/target/x86_64-linux-android/release/libserialrun.so" "$APK_TMP/lib/x86_64/"

# Copy resources
cp "$PROJECT_DIR/mobile/android/app/src/main/res/xml/device_filter.xml" "$APK_TMP/res/xml/"

# Compile Java to classes
echo "=== Compiling Java ==="
ANDROID_JAR="$ANDROID_SDK/platforms/android-34/android.jar"
[ -f "$ANDROID_JAR" ] || ANDROID_JAR=$(find "$ANDROID_SDK/platforms" -name "android.jar" | head -1)
echo "Using android.jar: $ANDROID_JAR"

JAVA_SRC="$PROJECT_DIR/mobile/android/app/src/main/java"
mkdir -p "$APK_TMP/classes"
javac -source 1.8 -target 1.8 -d "$APK_TMP/classes" \
  -bootclasspath "$ANDROID_JAR" \
  -classpath "$ANDROID_JAR" \
  "$JAVA_SRC/com/serialrun/mobile/SerialRunActivity.java"

# Convert classes to dex
echo "=== Converting to DEX ==="
D8_OUTPUT="$APK_TMP/dex_out"
mkdir -p "$D8_OUTPUT"
"$BUILD_TOOLS/d8.bat" --output "$D8_OUTPUT" --lib "$ANDROID_JAR" "$APK_TMP/classes/com/serialrun/mobile/SerialRunActivity.class" 2>&1 || \
  d8 --output "$D8_OUTPUT" --lib "$ANDROID_JAR" "$APK_TMP/classes/com/serialrun/mobile/SerialRunActivity.class"

# Compile resources with aapt2
echo "=== Compiling resources ==="
"$BUILD_TOOLS/aapt2.exe" compile --dir "$PROJECT_DIR/mobile/android/app/src/main/res" -o "$APK_TMP/compiled_res.zip" 2>&1

# Link resources
echo "=== Linking APK ==="
"$BUILD_TOOLS/aapt2.exe" link \
  -I "$ANDROID_JAR" \
  --manifest "$PROJECT_DIR/mobile/android/app/src/main/AndroidManifest.xml" \
  -A "$APK_TMP" \
  -o "$APK_TMP/unaligned.apk" \
  "$APK_TMP/compiled_res.zip" 2>&1

# Add classes.dex
echo "=== Adding DEX ==="
cp "$D8_OUTPUT/classes.dex" "$APK_TMP/"

# Repackage with dex
cd "$APK_TMP"
unzip -o unaligned.apk -d repackaged 2>&1 | tail -3
cp classes.dex repackaged/
rm -f unaligned.apk
# Use aapt2 to repackage
cd "$PROJECT_DIR"

# Or simpler: use aapt to add the dex
echo "=== Creating final APK ==="
APK_OUT="$BUILD_DIR/serialrun-mobile-v$VERSION.apk"
rm -f "$APK_OUT"
cd "$APK_TMP/repackaged"
zip -r "$APK_OUT" . 2>&1 | tail -3
cd "$PROJECT_DIR"

# Align the APK
echo "=== Aligning APK ==="
"$BUILD_TOOLS/zipalign.exe" -p 4 "$APK_OUT" "$APK_OUT.aligned"

# Sign the APK
echo "=== Signing APK ==="
KEYSTORE="$HOME/.android/debug.keystore"
[ -f "$KEYSTORE" ] || {
  mkdir -p "$HOME/.android"
  "$ANDROID_SDK/cmdline-tools/latest/bin/sdkmanager.bat" || true
  # Create a debug keystore using keytool
  keytool -genkey -v -keystore "$KEYSTORE" -storepass android -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US" 2>&1 | tail -3
}

"$BUILD_TOOLS/apksigner.bat" sign --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android --ks-key-alias androiddebugkey --out "$APK_OUT" "$APK_OUT.aligned" 2>&1 || \
  apksigner sign --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android --ks-key-alias androiddebugkey --out "$APK_OUT" "$APK_OUT.aligned" 2>&1

rm -f "$APK_OUT.aligned"

ls -la "$APK_OUT"
echo "=== Done ==="
