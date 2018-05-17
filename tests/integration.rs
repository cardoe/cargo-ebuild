extern crate assert_cli;

use std::fs::File;
use std::io::prelude::*;

// TODO: inspect use of tempdir/tempfile
//
#[test]
fn ebuild() {
    let ebuild_name = format!("cargo-ebuild-{}.ebuild", env!("CARGO_PKG_VERSION"));

    assert_cli::Assert::main_binary()
        .with_args(&["build"])
        .stdout()
        .is((format!("Wrote: {}", ebuild_name)).as_str())
        .unwrap();

    let mut new_file = match File::open(format!("{}", ebuild_name)) {
        Err(why) => panic!("couldn't open generated ebuild: {}", why),
        Ok(f) => f,
    };

    let mut new_ebuild = String::new();
    new_file.read_to_string(&mut new_ebuild).unwrap();

    let mut test_file = match File::open(format!("tests/{}", ebuild_name)) {
        Err(why) => panic!("couldn't open test ebuild: {}", why),
        Ok(f) => f,
    };

    let mut test_ebuild = String::new();
    test_file.read_to_string(&mut test_ebuild).unwrap();

    assert_eq!(new_ebuild, test_ebuild);
}
