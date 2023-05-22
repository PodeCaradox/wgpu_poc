
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
#[cfg(not(target_arch = "wasm32"))]
fn main()  {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("assets/");
    copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();

}