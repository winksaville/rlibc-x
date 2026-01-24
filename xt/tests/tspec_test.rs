use std::path::PathBuf;
use xt::tspec::{hash_spec, load_spec};
use xt::types::*;

fn test_data(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

#[test]
fn load_minimal_spec() {
    let spec = load_spec(&test_data("minimal.toml")).unwrap();

    assert_eq!(spec.cargo.len(), 1);
    assert_eq!(spec.cargo[0], CargoParam::Profile(Profile::Release));

    assert_eq!(spec.rustc.len(), 2);
    assert_eq!(spec.rustc[0], RustcParam::OptLevel(OptLevel::Oz));
    assert_eq!(spec.rustc[1], RustcParam::Panic(PanicStrategy::Abort));

    assert!(spec.linker.is_empty());
}

#[test]
fn load_spec_file_not_found() {
    let result = load_spec(&test_data("nonexistent.toml"));
    assert!(result.is_err());
}

#[test]
fn loaded_spec_hash_is_stable() {
    let spec = load_spec(&test_data("minimal.toml")).unwrap();
    let hash1 = hash_spec(&spec).unwrap();
    let hash2 = hash_spec(&spec).unwrap();
    assert_eq!(hash1, hash2);
}

fn lib_tspec(lib_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../libs")
        .join(lib_name)
        .join("tspec.toml")
}

#[test]
fn load_rlibc_x1_spec() {
    let spec = load_spec(&lib_tspec("rlibc-x1")).unwrap();

    assert!(spec.cargo.is_empty());
    assert!(spec.rustc.is_empty());
    assert_eq!(spec.linker.len(), 1);
    assert_eq!(
        spec.linker[0],
        LinkerParam::Args(vec!["-static".to_string(), "-nostartfiles".to_string()])
    );
}
