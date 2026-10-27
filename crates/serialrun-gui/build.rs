use std::path::Path;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_wechat_qr)");

    // The donation QR is a private asset: bundle it when present locally, but
    // keep CI and open-source builds independent of it.
    let qr = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/wechat_pay_qr.jpg");
    println!("cargo:rerun-if-changed={}", qr.display());
    if qr.exists() {
        println!("cargo:rustc-cfg=has_wechat_qr");
    }
}
