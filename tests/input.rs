use resumidor_rust::input::{list_txt_files, read_file};

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("resumidor_rust_test_{nanos}"));
    fs::create_dir(&dir).unwrap();
    dir
}

#[test]
fn test_list_txt_files_filters_case_insensitive() {
    let dir = make_temp_dir();
    let a = dir.join("a.txt");
    let b = dir.join("b.TXT");
    let c = dir.join("c.md");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();
    fs::write(&c, "c").unwrap();

    let got = list_txt_files(&dir).unwrap();
    let got: HashSet<_> = got.into_iter().collect();
    let expected: HashSet<_> = vec![a, b].into_iter().collect();
    assert_eq!(got, expected);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_read_file_reads_utf8() {
    let dir = make_temp_dir();
    let path = dir.join("hello.txt");
    fs::write(&path, "hola mundo").unwrap();

    let got = read_file(&path).unwrap();
    assert_eq!(got, "hola mundo");

    fs::remove_dir_all(&dir).unwrap();
}
