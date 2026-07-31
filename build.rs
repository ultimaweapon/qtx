use std::process::Command;

use cmake::Config;

fn main() {
    println!("cargo::rerun-if-env-changed=DOCS_RS");

    // Do nothing if running from Docs.rs.
    let mut cmake = match std::env::var_os("DOCS_RS") {
        Some(_) => return,
        None => Config::new(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()),
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
        let qmake = Command::new("qtpaths6")
            .arg("--query")
            .arg("QT_INSTALL_LIBS")
            .output()
            .unwrap();

        assert!(qmake.status.success());

        // Add to search path.
        let libs = std::str::from_utf8(&qmake.stdout).unwrap();

        println!("cargo::rustc-link-search=framework={libs}");
    }
}
