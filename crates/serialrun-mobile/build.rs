//! Build script for serialrun-mobile
//!
//! Handles Android NDK configuration and linking.

fn main() {
    // On Android, we need to link against the Android system libraries
    #[cfg(target_os = "android")]
    {
        println!("cargo:rustc-link-lib=dylib=android");
        println!("cargo:rustc-link-lib=dylib=log");
        println!("cargo:rustc-link-lib=dylib=OpenSLES");
    }
}
