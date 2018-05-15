extern crate assert_cli;

use std::fs::File;
use std::io::prelude::*;

// TODO: inspect use of tempdir/tempfile
//
#[test]
fn ebuild() {

    assert_cli::Assert::main_binary()
        .stdout().is("Wrote: cargo-ebuild-0.1.7.ebuild")
        .unwrap();

    let mut new_file = File::open("cargo-ebuild-0.1.7.ebuild").unwrap.or_else(|| {
        panic!("couldn't open generated ebuild: {}", why.description());
    });
    let mut new_ebuild = String::new();
    new_file.read_to_string(&mut new_ebuild).unwrap();

    let mut test_file = File::open("tests/cargo-ebuild-0.1.7.ebuild").unwrap.or_else(|| {
        panic!("couldn't open test ebuild: {}", why.description());
    });
    let mut test_ebuild = String::new();
    test_file.read_to_string(&mut test_ebuild).unwrap();


    assert_eq!(new_ebuild, test_ebuild);

}
