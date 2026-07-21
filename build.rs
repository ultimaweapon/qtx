use cmake::Config;

fn main() {
    // Do nothing if running from Docs.rs.
    let mut cmake = match std::env::var_os("DOCS_RS") {
        Some(_) => return,
        None => Config::new(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()),
    };

    // Build FFI.
    let out = cmake.build();

    println!("cargo::rustc-link-search=native={}", out.to_str().unwrap());
    println!("cargo::rustc-link-lib=static=qtx");
}
