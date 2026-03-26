extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    cbindgen::generate(crate_dir)
        .expect("Unable to generate the bindings orz")
        .write_to_file(&output_file);
}
