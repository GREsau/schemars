use std::env;
use std::process::Command;
use std::str::{self, FromStr};

// Based on https://github.com/serde-rs/serde/blob/master/serde/build.rs

fn main() {
    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    let target = env::var("TARGET").unwrap();
    let emscripten = target == "asmjs-unknown-emscripten" || target == "wasm32-unknown-emscripten";

    // Atomic types, and non-zero signed integers stabilized in Rust 1.34:
    // https://blog.rust-lang.org/2019/04/11/Rust-1.34.0.html#library-stabilizations
    if minor >= 34 {
        println!("cargo:rustc-cfg=num_nonzero_signed");

        // Whitelist of archs that support std::sync::atomic module. Ideally we
        // would use #[cfg(target_has_atomic = "...")] but it is not stable yet.
        // Instead this is based on rustc's src/librustc_target/spec/*.rs.
        let has_atomic64 = target.starts_with("x86_64")
            || target.starts_with("i686")
            || target.starts_with("aarch64")
            || target.starts_with("powerpc64")
            || target.starts_with("sparc64")
            || target.starts_with("mips64el");
        let has_atomic32 = has_atomic64 || emscripten;
        if has_atomic64 {
            println!("cargo:rustc-cfg=std_atomic64");
        }
        if has_atomic32 {
            println!("cargo:rustc-cfg=std_atomic");
        }
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }

    let next = match pieces.next() {
        Some(next) => next,
        None => return None,
    };

    u32::from_str(next).ok()
}
