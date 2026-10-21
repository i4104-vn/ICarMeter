fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" {
        println!("cargo:rerun-if-changed=compat/glibc_compat.c");
        let rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
        let cargo_encoded = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
        if !rustflags.contains("glibc_compat.o") && !cargo_encoded.contains("glibc_compat.o") {
            let out_dir = match std::env::var("OUT_DIR") {
                Ok(val) => val,
                Err(_) => return,
            };
            let obj_path = format!("{}/glibc_compat.o", out_dir);
            let cc = std::env::var("CC").unwrap_or_else(|_| "gcc".to_string());
            let status = std::process::Command::new(&cc)
                .args(&["-O2", "-fPIC", "-c", "compat/glibc_compat.c", "-o", &obj_path])
                .status();

            if let Ok(s) = status {
                if s.success() {
                    println!("cargo:rustc-link-arg={}", obj_path);
                }
            }
        }
    }
}
