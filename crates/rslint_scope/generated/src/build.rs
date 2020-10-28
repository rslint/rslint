fn main() {
    #[cfg(not(windows))]
    libtool();
}

#[cfg(not(windows))]
fn libtool() {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    let topdir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fbufpath = format!("{}/src/flatbuf.rs", topdir);

    /* Only include flatbuf files as dependencies if the project was
     * compiled with flatbuf support.
     */
    if Path::new(fbufpath.as_str()).exists() {
        println!("cargo:rerun-if-changed=src/flatbuf.rs");
        println!("cargo:rerun-if-changed=src/flatbuf_generated.rs");
    }
    println!("cargo:rerun-if-changed=src/api.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/ovsdb.rs");
    println!("cargo:rerun-if-changed=src/update_handler.rs");

    let lib = "librslint_scoping_ddlog";

    /* Start: fixup for a bug in libtool, which does not correctly
     * remove the symlink it creates.  Remove this fixup once an updated
     * libtool crate is available.
     *
     * See: https://github.com/kanru/libtool-rs/issues/2#issue-440212008
     */
    let topdir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let target_dir = format!("{}/target/{}", topdir, profile);
    let libs_dir = format!("{}/.libs", target_dir);
    let new_lib_path = PathBuf::from(format!("{}/{}.a", libs_dir, lib));
    let _ = fs::remove_file(&new_lib_path);
    /* End: fixup */

    libtool::generate_convenience_lib(lib).unwrap();
}
