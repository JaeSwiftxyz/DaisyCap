fn main() {
    // Embed Info.plist into the Mach-O so TCC finds usage strings and a stable
    // bundle identifier even when the binary runs outside the .app bundle.
    let plist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("support/Info.plist");
    println!("cargo:rerun-if-changed={}", plist.display());
    println!("cargo:rustc-link-arg-bins=-Wl,-sectcreate,__TEXT,__info_plist,{}", plist.display());
    // whisper.cpp's Metal backend guards newer APIs with @available, which compiles to calls into
    // clang's runtime when the deployment target is older than those APIs. rustc links with
    // -nodefaultlibs, so that runtime has to be named here.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let out = std::process::Command::new("xcrun").args(["clang", "--print-resource-dir"]).output();
        if let Some(out) = out.ok().filter(|o| o.status.success()) {
            let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let rt = std::path::Path::new(&dir).join("lib/darwin/libclang_rt.osx.a");
            if rt.exists() {
                println!("cargo:rustc-link-arg={}", rt.display());
            }
        }
    }
}
