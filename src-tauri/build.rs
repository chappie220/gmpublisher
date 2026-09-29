fn main() {
	if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
		// Look for libsteam_api.so next to the executable, or in /usr/lib/gmpublisher when installed from a .rpm/.deb package
		println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN:$ORIGIN/../lib/gmpublisher");
	}

	tauri_build::build()
}
