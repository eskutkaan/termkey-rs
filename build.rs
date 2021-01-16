use std::process::Command;

fn main() {
    let git_ref = "v0.17";
    assert!(Command::new("./build_termkey.sh")
        .arg(git_ref)
        .status()
        .unwrap()
        .success());
    println!(
        "cargo:rustc-flags= -L {}/termkey-c/{}/.libs -l termkey",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        git_ref
    );
}
