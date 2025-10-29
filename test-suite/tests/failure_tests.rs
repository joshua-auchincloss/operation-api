use operation_api_fs::memory;
use operation_api_test_suite::*;

#[tokio::test]
async fn test_fail_01_missing_dependency() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/missing_dep_use.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_01_missing_dependency",
        "Missing Dependency",
        "Verify compilation fails when a used dependency is not declared",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("external_pkg")
            || err_msg.contains("not found")
            || err_msg.contains("dependency"),
        "Error should mention missing dependency: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_02_circular_dependency() {
    let fs = memory! {
        "pkg-a/schema.toml" => r#"[package]
name = "pkg-a"
version = "1.0.0"

[dependencies]
pkg-b = { path = "../pkg-b" }
"#,
        "pkg-a/schema/lib.pld" => "namespace pkg_a;\nuse pkg_b;",
        "pkg-b/schema.toml" => r#"[package]
name = "pkg-b"
version = "1.0.0"

[dependencies]
pkg-a = { path = "../pkg-a" }
"#,
        "pkg-b/schema/lib.pld" => "namespace pkg_b;\nuse pkg_a;",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_02_circular_dependency",
        "Circular Dependency",
        "Detect and reject circular dependency chains",
        false,
        vec![Tag::Validations],
    );
    harness.set_root("pkg-a");

    let err = harness.compile_fail().await;

    // Verify error mentions circular dependency
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("circular") || err_msg.contains("cycle"),
        "Error should mention circular dependency: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_03_invalid_checksum() {
    let fs = memory! {
        "dep/schema.toml" => include_str!("../fragments/dep_manifest.toml"),
        "dep/schema/lib.pld" => include_str!("../fragments/dep_lib.pld"),
        "pkg/schema.toml" => r#"[package]
name = "pkg"
version = "1.0.0"

[dependencies]
dep = { path = "../dep" }
"#,
        "pkg/schema.lock" => r#"# This lockfile has an INCORRECT checksum
[[package]]
name = "dep"
version = "1.0.0"
source = { path = "../dep" }
checksum = "deadbeefdeadbeefdeadbeefdeadbeef"

[provides]
dep = ["dep"]
"#,
        "pkg/schema/lib.pld" => "namespace pkg;\nuse dep;",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_03_invalid_checksum",
        "Invalid Lockfile Checksum",
        "Verify compilation detects modified dependencies and regenerates lockfile",
        true, // Should succeed but regenerate lockfile
        vec![Tag::Lockfile],
    );

    let _ = harness.compile_pass().await;
    let write_ops = harness.count_operations("Write");
    assert!(
        write_ops > 0,
        "Should have rewritten lockfile with correct checksum"
    );
}

#[tokio::test]
async fn test_fail_04_version_conflict() {
    let fs = memory! {
        "lib/schema.toml" => r#"[package]
name = "lib"
version = "2.0.0"
"#,
        "lib/schema/lib.pld" => "namespace lib;\nstruct Item { id: u64 }",
        "pkg-a/schema.toml" => r#"[package]
name = "pkg-a"
version = "1.0.0"

[dependencies]
lib = { path = "../lib", version = "^1.0" }
"#,
        "pkg-a/schema/lib.pld" => "namespace pkg_a;\nuse lib;",
        "pkg-b/schema.toml" => r#"[package]
name = "pkg-b"
version = "1.0.0"

[dependencies]
lib = { path = "../lib", version = "^2.0" }
"#,
        "pkg-b/schema/lib.pld" => "namespace pkg_b;\nuse lib;",
        "app/schema.toml" => r#"[package]
name = "app"
version = "1.0.0"

[dependencies]
pkg-a = { path = "../pkg-a" }
pkg-b = { path = "../pkg-b" }
"#,
        "app/schema/lib.pld" => "namespace app;\nuse pkg_a;\nuse pkg_b;",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_04_version_conflict",
        "Version Conflict - Incompatible Requirements",
        "Detect when dependencies require incompatible versions",
        false,
        vec![Tag::Validations],
    );
    harness.set_root("app");

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("version")
            || err_msg.contains("conflict")
            || err_msg.contains("incompatible"),
        "Error should mention version conflict: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_05_missing_namespace() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/missing_namespace_use.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_05_missing_namespace",
        "Missing Namespace File",
        "Verify error when imported namespace doesn't exist",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("missing_namespace") || err_msg.contains("not found"),
        "Error should mention missing namespace: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_06_undefined_type() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/undefined_type.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_06_undefined_type",
        "Type Reference to Undefined Type",
        "Catch references to non-existent types",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("UndefinedType")
            || err_msg.contains("not found")
            || err_msg.contains("undefined"),
        "Error should mention undefined type: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_07_duplicate_type() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/duplicate_type.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_07_duplicate_type",
        "Duplicate Type Definition",
        "Prevent conflicting type definitions in same namespace",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("duplicate")
            || err_msg.contains("Foo")
            || err_msg.contains("already defined"),
        "Error should mention duplicate type: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_08_invalid_enum_discriminant() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/invalid_enum_discriminant.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_08_invalid_enum_discriminant",
        "Invalid Enum Discriminant",
        "Reject invalid enum values",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("1.5")
            || err_msg.contains("invalid")
            || err_msg.contains("discriminant")
            || err_msg.contains("float"),
        "Error should mention invalid discriminant: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_09_malformed_manifest() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/malformed_manifest.toml"),
        "pkg/schema/lib.pld" => "namespace pkg;",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_09_malformed_manifest",
        "Malformed Manifest",
        "Catch syntax errors in TOML manifests",
        false,
        vec![Tag::Soundness],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("TOML") || err_msg.contains("parse") || err_msg.contains("syntax"),
        "Error should mention TOML parse error: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_fail_10_wrong_import_syntax() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/wrong_import_syntax.pld"),
        "pkg/schema/internal.pld" => include_str!("../fragments/internal_namespace.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_fail_10_wrong_import_syntax",
        "Import from Same Package with Wrong Path",
        "Verify imports within package use correct syntax",
        false,
        vec![Tag::Validations],
    );

    let err = harness.compile_fail().await;
    let err_msg = format!("{:?}", err);
    assert!(
        !err_msg.is_empty(),
        "Should produce an error for incorrect import syntax"
    );
}
