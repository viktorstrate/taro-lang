// Test generation build tool is inspired by this blog post
// https://blog.cyplo.dev/posts/2018/12/generate-rust-tests-from-data/

use std::env;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// build script's entry point
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("transpilation_tests.rs");
    let mut test_file = File::create(&destination).unwrap();

    // write test file header, put `use`, `const` etc there
    write_header(&mut test_file);

    let test_data_files = read_dir("./tests/transpilations/").unwrap();

    for entry in test_data_files {
        write_test(&mut test_file, &entry.unwrap());
    }
}

fn write_test(test_file: &mut File, entry: &DirEntry) {
    let directory = entry.path().canonicalize().unwrap();
    let path = directory.display();
    let test_name = format!(
        "transpile_{}",
        directory.file_stem().unwrap().to_string_lossy()
    );

    write!(
        test_file,
        include_str!("./tests/transpile_template.txt"),
        name = test_name,
        path = path
    )
    .unwrap();
}

fn write_header(test_file: &mut File) {
    write!(
        test_file,
        r#"use taro::ir::test_utils::utils::final_codegen;

"#
    )
    .unwrap();
}
