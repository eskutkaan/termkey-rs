#![allow(unstable)]

use std::os;
use std::old_io::Command;
use std::old_io::process::InheritFd;

fn main() {
    assert!(Command::new("./build_termkey.sh").
            stdout(InheritFd(1)).stderr(InheritFd(2)).
            status().unwrap().success());
    println!("cargo:rustc-flags= -L {}/termkey-c/.libs -l termkey",
      os::getenv("CARGO_MANIFEST_DIR").unwrap());
}
