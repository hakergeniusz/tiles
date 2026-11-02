// Compiles the bundled GResource (app icon) at build time.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let target = out_dir.join("tiles.gresource");
    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir=resources")
        .arg("--internal")
        .arg("--target")
        .arg(&target)
        .arg("resources/tiles.gresource.xml")
        .status()
        .expect("glib-compile-resources must be installed (glib2 / libglib2.0-dev)");
    assert!(status.success(), "glib-compile-resources failed");

    println!("cargo:rerun-if-changed=resources/tiles.gresource.xml");
    println!("cargo:rerun-if-changed=resources/tiles.svg");
}
