use gtfsfabrik::errors::osm::OSMError;
use gtfsfabrik::files::osm::validate_osm;

use std::fs;
use tempfile::tempdir;

const VALID_PBF: &str = "/home/aaron/projects/gtfsfabrik/tests/inputs/files/example_osm.pbf";

#[test]
fn valid_pbf_passes() {
    assert!(validate_osm(VALID_PBF).is_ok());
}

#[test]
fn wrong_extension_osm_fails() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("example_osm.json");
    fs::write(&f, b"").unwrap();
    assert!(matches!(
        validate_osm(f.to_str().unwrap()),
        Err(OSMError::NotAPbfFile(_))
    ));
}

#[test]
fn directory_returns_not_a_file() {
    let dir = tempdir().unwrap();
    assert!(matches!(
        validate_osm(dir.path().to_str().unwrap()),
        Err(OSMError::NotAFile(_))
    ));
}

#[test]
fn cant_find_file() {
    assert!(matches!(
        validate_osm("/tmp/does_not_exist"),
        Err(OSMError::FileNotFound(_))
    ));
}

#[test]
fn nonexistent_pbf_path() {
    let result = validate_osm("/tmp/some_fake_file.pbf");
    // this currently returns Ok(()) which is wrong -- existence check
    // needs to come before the extension check in validate_osm
    assert!(result.is_err()); // will fail until bug is fixed
}
