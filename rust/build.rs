use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../src/can_ota.c");
    println!("cargo:rerun-if-changed=../include/sdm/can_ota.h");

    cc::Build::new()
        .file("../src/can_ota.c")
        .include("../include")
        .compile("can_ota");

    let bindings = bindgen::Builder::default()
        .header("../include/sdm/can_ota.h")
        .allowlist_item("(can_ota.*|CAN_OTA.*)")
        .newtype_enum("can_ota_result|can_ota_state")
        .use_core()
        .generate()
        .expect("bindgen failed on can_ota.h");
    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("write bindings.rs");
}
