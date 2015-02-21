 #![feature(env)]
 #![feature(process)]

use std::process::Command;

fn main() {
    assert!(Command::new("./build_termkey.sh").
            status().unwrap().success());
    println!("cargo:rustc-flags= -L {}/termkey-c/.libs -l termkey",
      std::env::var("CARGO_MANIFEST_DIR").unwrap());
}
