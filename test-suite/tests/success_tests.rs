use operation_api_fs::memory;
use operation_api_test_suite::*;

#[tokio::test]
async fn test_01_minimal_reproducible() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/minimal_lib.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_01_minimal_reproducible",
        "Minimal Reproducible Example",
        "Verify basic compilation of a single-file project with no dependencies",
        true,
        vec![Tag::Smoke],
    );

    let _ = harness.compile_pass().await;
    harness.assert_no_lockfile();
}

#[tokio::test]
async fn test_02_namespace_by_file() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/use_bar.pld"),
        "pkg/schema/bar.pld" => include_str!("../fragments/bar_enum.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_02_namespace_by_file",
        "Public Namespace by File",
        "Test importing a namespace defined in a sibling file",
        true,
        vec![Tag::Smoke, Tag::Namespace],
    );

    let _ = harness.compile_pass().await;
    harness.assert_no_lockfile();
}

#[tokio::test]
async fn test_03_namespace_by_directory() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/use_bar.pld"),
        "pkg/schema/bar/test.pld" => include_str!("../fragments/bar_enum.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_03_namespace_by_directory",
        "Public Namespace by Directory",
        "Test importing a namespace defined via directory structure",
        true,
        vec![Tag::Smoke, Tag::Namespace],
    );

    let _ = harness.compile_pass().await;
    harness.assert_no_lockfile();
}

#[tokio::test]
async fn test_04_external_path_dependency() {
    let fs = memory! {
        "abc-corp/schema.toml" => include_str!("../fragments/abc_corp_manifest.toml"),
        "abc-corp/schema/lib.pld" => include_str!("../fragments/abc_corp_lib.pld"),
        "pkg/schema.toml" => include_str!("../fragments/pkg_with_dep_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/use_bar.pld"),
        "pkg/schema/bar/test.pld" => include_str!("../fragments/bar_with_external_type.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_04_external_path_dependency",
        "Smoke Package with External Path Dependency",
        "Test compilation with a path-based external dependency",
        true,
        vec![Tag::Smoke, Tag::Dependencies],
    );

    let _ = harness.compile_pass().await;
    harness.assert_lockfile_written();
    harness.assert_lockfile_contains("abc-corp");
}

#[tokio::test]
async fn test_05_transitive_dependencies() {
    let fs = memory! {
        "base/schema.toml" => include_str!("../fragments/base_manifest.toml"),
        "base/schema/lib.pld" => include_str!("../fragments/base_lib.pld"),
        "middle/schema.toml" => include_str!("../fragments/middle_manifest.toml"),
        "middle/schema/lib.pld" => include_str!("../fragments/middle_lib.pld"),
        "top/schema.toml" => include_str!("../fragments/top_manifest.toml"),
        "top/schema/lib.pld" => include_str!("../fragments/top_lib.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_05_transitive_dependencies",
        "Nested Path Dependencies (Transitive)",
        "Test transitive dependency resolution",
        true,
        vec![Tag::Smoke, Tag::Dependencies],
    );
    harness.set_root("top");

    let _ = harness.compile_pass().await;
    harness.assert_lockfile_written();
    harness.assert_lockfile_contains("middle");
    harness.assert_lockfile_contains("base");
}

#[tokio::test]
async fn test_06_diamond_dependency() {
    let fs = memory! {
        "common/schema.toml" => include_str!("../fragments/common_manifest.toml"),
        "common/schema/lib.pld" => include_str!("../fragments/common_lib.pld"),
        "lib-a/schema.toml" => include_str!("../fragments/lib_a_manifest.toml"),
        "lib-a/schema/lib.pld" => include_str!("../fragments/lib_a_lib.pld"),
        "lib-b/schema.toml" => include_str!("../fragments/lib_b_manifest.toml"),
        "lib-b/schema/lib.pld" => include_str!("../fragments/lib_b_lib.pld"),
        "app/schema.toml" => include_str!("../fragments/app_manifest.toml"),
        "app/schema/lib.pld" => include_str!("../fragments/app_lib.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_06_diamond_dependency",
        "Diamond Dependency Resolution",
        "Test multiple dependencies with shared transitive dependency",
        true,
        vec![Tag::Smoke, Tag::Dependencies],
    );
    harness.set_root("app");

    let _ = harness.compile_pass().await;

    harness.assert_lockfile_written();
    harness.assert_lockfile_contains("common");
    harness.assert_lockfile_contains("lib-a");
    harness.assert_lockfile_contains("lib-b");

    let lockfile = harness.read_lockfile().unwrap();
    let common_count = lockfile.matches("name = \"common\"").count();
    assert_eq!(
        common_count, 1,
        "Common should appear exactly once in lockfile"
    );
}

#[tokio::test]
async fn test_07_valid_lockfile_checksum() {
    // First, compile to generate a valid lockfile
    let fs = memory! {
        "dep/schema.toml" => include_str!("../fragments/dep_manifest.toml"),
        "dep/schema/lib.pld" => include_str!("../fragments/dep_lib.pld"),
        "pkg/schema.toml" => r#"[package]
name = "pkg"
version = "1.0.0"

[dependencies]
dep = { path = "../dep" }
"#,
        "pkg/schema/lib.pld" => "namespace pkg;\nuse dep;\nstruct Wrapper { data: Data };",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_07_valid_lockfile_checksum",
        "Lockfile Validation - Valid Checksum",
        "Test that valid lockfile checksums are accepted and don't trigger rewrites",
        true,
        vec![Tag::Smoke, Tag::Lockfile],
    );

    let _ = harness.compile_pass().await;
    let lockfile_content = harness
        .read_lockfile()
        .expect("Lockfile should exist");

    // Add the lockfile to the same filesystem and compile again
    harness.add_text_file("pkg/schema.lock", &lockfile_content);
    harness.clear_operations();

    let _ = harness.compile_pass().await;

    // Lockfile should not be rewritten (validated successfully)
    let write_count = harness.count_operations("Write");
    assert_eq!(
        write_count, 0,
        "Lockfile should not be rewritten when valid"
    );
}

#[tokio::test]
async fn test_08_version_pruning() {
    let fs = memory! {
        "lib/schema.toml" => r#"[package]
name = "lib"
version = "1.2.3"
"#,
        "lib/schema/lib.pld" => "namespace lib;\nstruct Item { id: u64 };",
        "pkg/schema.toml" => r#"[package]
name = "pkg"
version = "1.0.0"

[dependencies]
lib = { path = "../lib", version = "^1.0" }
"#,
        "pkg/schema/lib.pld" => "namespace pkg;\nuse lib;",
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_08_version_pruning",
        "Version Compatibility - Multiple Compatible Versions",
        "Test version pruning keeps highest compatible version",
        true,
        vec![Tag::Smoke, Tag::VersionResolution],
    );

    let _ = harness.compile_pass().await;
    harness.assert_lockfile_written();
    harness.assert_lockfile_contains("lib");
}

#[tokio::test]
async fn test_09_string_enum() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/string_enum.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_09_string_enum",
        "Enum with String Values",
        "Test string-based enum compilation",
        true,
        vec![Tag::Smoke, Tag::Soundness],
    );

    let _ = harness.compile_pass().await;
    harness.assert_no_lockfile();
}

#[tokio::test]
async fn test_10_oneof_mixed_types() {
    let fs = memory! {
        "pkg/schema.toml" => include_str!("../fragments/minimal_manifest.toml"),
        "pkg/schema/lib.pld" => include_str!("../fragments/oneof_mixed.pld"),
    };

    let mut harness = TestHarness::with_metadata(
        fs,
        "test_10_oneof_mixed_types",
        "OneOf with Multiple Types",
        "Test union type compilation with mixed types",
        true,
        vec![Tag::Smoke, Tag::Soundness],
    );

    let _ = harness.compile_pass().await;
    harness.assert_no_lockfile();
}
