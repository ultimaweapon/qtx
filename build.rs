use std::path::PathBuf;
use std::process::Command;

use cmake::Config;

fn main() {
    println!("cargo::rerun-if-env-changed=DOCS_RS");

    // Do nothing if running from Docs.rs.
    let mut cmake = match std::env::var_os("DOCS_RS") {
        Some(_) => return,
        None => Config::new(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()),
    };

    // Check for Qt6_DIR variable.
    let qtpaths = match std::env::var_os("QTX_QT_PATH") {
        Some(v) => {
            cmake.define("CMAKE_PREFIX_PATH", v.as_os_str());

            // Build path to qtpaths6. This need to be a full path.
            let mut v = std::fs::canonicalize(v).unwrap();

            v.push("bin");
            v.push("qtpaths6");

            v
        }
        None => PathBuf::from("qtpaths6"),
    };

    // Build FFI.
    let mut out = cmake.build();

    out.push("lib");

    println!("cargo::rustc-link-search=native={}", out.to_str().unwrap());
    println!("cargo::rustc-link-lib=static=qtx");

    // Link Qt.
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let add_path = match target.as_str() {
        "linux" => {
            println!("cargo::rustc-link-lib=Qt6Core");
            println!("cargo::rustc-link-lib=Qt6Gui");
            println!("cargo::rustc-link-lib=Qt6Widgets");
            println!("cargo::rustc-link-lib=stdc++");

            false
        }
        "macos" => {
            println!("cargo::rustc-link-lib=framework=QtCore");
            println!("cargo::rustc-link-lib=framework=QtGui");
            println!("cargo::rustc-link-lib=framework=QtWidgets");
            println!("cargo::rustc-link-lib=c++");

            true
        }
        _ => todo!(),
    };

    if add_path {
        // Get path for Qt libraries.
        let qmake = Command::new(qtpaths)
            .arg("--query")
            .arg("QT_INSTALL_LIBS")
            .output()
            .unwrap();

        assert!(qmake.status.success());

        // Add to search path.
        let libs = std::str::from_utf8(&qmake.stdout).unwrap();

        if target == "macos" {
            println!("cargo::rustc-link-search=framework={libs}");
        } else {
            println!("cargo::rustc-link-search=native={libs}");
        }
    }
}
