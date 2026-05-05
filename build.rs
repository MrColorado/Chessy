fn main() {
    let version = std::env::var("VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=APP_VERSION={version}");
    println!("cargo:rerun-if-env-changed=VERSION");
}
