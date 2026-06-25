#!/usr/bin/env python3
"""Build SerialRUN Mobile APK from scratch using Android SDK tools."""

import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

PROJECT_DIR = Path(__file__).resolve().parent.parent
ANDROID_SDK = Path("C:/Users/yao/android-sdk")
BUILD_TOOLS = ANDROID_SDK / "build-tools/34.0.0"
NDK = ANDROID_SDK / "ndk/26.1.10909125"
PLATFORM = ANDROID_SDK / "platforms/android-34/android.jar"
BUILD_DIR = PROJECT_DIR / "apk_build"
APK_TMP = BUILD_DIR / "apk_tmp"
APK_OUT = BUILD_DIR / "serialrun-mobile-v0.1.0.apk"
VERSION = "0.1.0"

# Add NDK clang to PATH for cargo
NDK_BIN = NDK / "toolchains/llvm/prebuilt/windows-x86_64/bin"
os.environ["PATH"] = str(NDK_BIN) + os.pathsep + os.environ.get("PATH", "")


def run(cmd, **kwargs):
    """Run command and print it."""
    print(f">>> {cmd}")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, **kwargs)
    if result.returncode != 0:
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
    return result


def build_rust():
    """Build Rust libraries for both Android architectures."""
    print("\n=== Building Rust libraries ===")
    os.chdir(PROJECT_DIR)
    # arm64
    r = run("cargo build --release --target aarch64-linux-android -p serialrun-mobile-app --lib")
    if r.returncode != 0:
        sys.exit("Cargo build failed")
    # x86_64
    r = run("cargo build --release --target x86_64-linux-android -p serialrun-mobile-app --lib")
    if r.returncode != 0:
        sys.exit("Cargo build failed")


def setup_structure():
    """Set up the APK directory structure."""
    print("\n=== Setting up APK structure ===")
    if APK_TMP.exists():
        shutil.rmtree(APK_TMP)
    (APK_TMP / "lib/arm64-v8a").mkdir(parents=True)
    (APK_TMP / "lib/x86_64").mkdir(parents=True)
    (APK_TMP / "res/xml").mkdir(parents=True)
    (APK_TMP / "META-INF").mkdir(parents=True)

    # Copy .so files
    shutil.copy(
        PROJECT_DIR / "target/aarch64-linux-android/release/libserialrun.so",
        APK_TMP / "lib/arm64-v8a/libserialrun.so",
    )
    shutil.copy(
        PROJECT_DIR / "target/x86_64-linux-android/release/libserialrun.so",
        APK_TMP / "lib/x86_64/libserialrun.so",
    )

    # Copy resources
    res_xml = PROJECT_DIR / "mobile/android/app/src/main/res/xml/device_filter.xml"
    if res_xml.exists():
        shutil.copy(res_xml, APK_TMP / "res/xml/device_filter.xml")

    print(f"  arm64-v8a lib: {(APK_TMP / 'lib/arm64-v8a/libserialrun.so').stat().st_size // 1024} KB")
    print(f"  x86_64 lib:    {(APK_TMP / 'lib/x86_64/libserialrun.so').stat().st_size // 1024} KB")


def compile_java():
    """Compile Java to .class files."""
    print("\n=== Compiling Java ===")
    java_src = PROJECT_DIR / "mobile/android/app/src/main/java"
    classes_dir = APK_TMP / "classes"
    classes_dir.mkdir(exist_ok=True)
    r = run(
        f'javac -source 1.8 -target 1.8 -d "{classes_dir}" '
        f'-bootclasspath "{PLATFORM}" '
        f'"{java_src}/com/serialrun/mobile/SerialRunActivity.java"'
    )
    if r.returncode != 0:
        sys.exit("javac failed")


def convert_dex():
    """Convert .class to classes.dex using d8."""
    print("\n=== Converting to DEX ===")
    dex_out = APK_TMP / "dex_out"
    dex_out.mkdir(exist_ok=True)
    classes_dir = APK_TMP / "classes"
    r = run(
        f'"{BUILD_TOOLS}/d8.bat" --output "{dex_out}" --lib "{PLATFORM}" '
        f'"{classes_dir}/com/serialrun/mobile/SerialRunActivity.class"'
    )
    if r.returncode != 0:
        # Try without .bat
        r = run(
            f'd8 --output "{dex_out}" --lib "{PLATFORM}" '
            f'"{classes_dir}/com/serialrun/mobile/SerialRunActivity.class"'
        )
        if r.returncode != 0:
            sys.exit("d8 failed")
    shutil.copy(dex_out / "classes.dex", APK_TMP / "classes.dex")


def link_apk():
    """Link resources and create the APK (no lib files yet)."""
    print("\n=== Linking APK ===")
    # Compile resources
    res_src = PROJECT_DIR / "mobile/android/app/src/main/res"
    compiled_res = APK_TMP / "compiled_res.zip"
    r = run(
        f'"{BUILD_TOOLS}/aapt2.exe" compile --dir "{res_src}" -o "{compiled_res}"'
    )
    if r.returncode != 0:
        sys.exit("aapt2 compile failed")

    # Link (no -A flag here, we add lib and dex manually)
    manifest = PROJECT_DIR / "mobile/android/app/src/main/AndroidManifest.xml"
    linked_apk = APK_TMP / "linked.apk"
    r = run(
        f'"{BUILD_TOOLS}/aapt2.exe" link '
        f'-I "{PLATFORM}" '
        f'--manifest "{manifest}" '
        f'-o "{linked_apk}" '
        f'"{compiled_res}"'
    )
    if r.returncode != 0:
        sys.exit("aapt2 link failed")
    return linked_apk


def add_files_to_apk(linked_apk):
    """Add classes.dex and lib files at the right paths."""
    print("\n=== Adding DEX and libs to APK ===")
    with zipfile.ZipFile(linked_apk, "a") as z:
        # Add classes.dex
        z.write(APK_TMP / "classes.dex", "classes.dex")
        # Add lib files at the right path
        for arch in ("arm64-v8a", "x86_64"):
            lib_path = APK_TMP / f"lib/{arch}/libserialrun.so"
            if lib_path.exists():
                z.write(lib_path, f"lib/{arch}/libserialrun.so")
                print(f"  Added lib/{arch}/libserialrun.so")


def sign_apk(apk_file):
    """Sign the APK with a debug keystore."""
    print("\n=== Signing APK ===")
    keystore = Path.home() / ".android/debug.keystore"
    if not keystore.exists():
        keystore.parent.mkdir(parents=True, exist_ok=True)
        r = run(
            f'keytool -genkey -v -keystore "{keystore}" -storepass android '
            f'-alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 '
            f'-validity 10000 -dname "CN=Android Debug,O=Android,C=US"'
        )
        if r.returncode != 0:
            sys.exit("keytool failed")

    # zipalign
    aligned = APK_TMP / "aligned.apk"
    r = run(f'"{BUILD_TOOLS}/zipalign.exe" -p -f 4 "{apk_file}" "{aligned}"')
    if r.returncode != 0:
        sys.exit("zipalign failed")

    # sign
    if APK_OUT.exists():
        APK_OUT.unlink()
    r = run(
        f'"{BUILD_TOOLS}/apksigner.bat" sign --ks "{keystore}" '
        f'--ks-pass pass:android --key-pass pass:android '
        f'--ks-key-alias androiddebugkey --out "{APK_OUT}" "{aligned}"'
    )
    if r.returncode != 0:
        # Try without .bat
        r = run(
            f'apksigner sign --ks "{keystore}" '
            f'--ks-pass pass:android --key-pass pass:android '
            f'--ks-key-alias androiddebugkey --out "{APK_OUT}" "{aligned}"'
        )
        if r.returncode != 0:
            sys.exit("apksigner failed")
    print(f"  Signed APK: {APK_OUT}")
    print(f"  Size: {APK_OUT.stat().st_size // 1024} KB")


def main():
    build_rust()
    setup_structure()
    compile_java()
    convert_dex()
    linked = link_apk()
    add_files_to_apk(linked)
    sign_apk(linked)
    print("\n=== APK build complete ===")
    print(f"  {APK_OUT}")


if __name__ == "__main__":
    main()
