fn main() {
    #[cfg(target_os = "macos")]
    build_macos();
}

fn build_macos() {
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11");
}
