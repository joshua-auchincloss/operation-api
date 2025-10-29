/// Normalize a package name from import format (abc_foo) to manifest format (abc-foo)
///
/// Package manifests always use kebab-case (abc-foo), while imports use snake_case (abc_foo).
/// This function converts from import format to package format for dependency resolution.
///
/// **Internal Convention**: All internal tracking (dependencies map, loaded_versions, processing_set,
/// referenced items, CompilationTask.package_name, dependency chains) uses import format (snake_case).
/// Only convert to package format (kebab-case) when accessing manifest.dependencies.
pub fn normalize_import_to_package_name(import_name: &str) -> String {
    import_name.replace('_', "-")
}

/// Normalize a package name from manifest format (abc-foo) to import format (abc_foo)
///
/// This is the inverse of `normalize_import_to_package_name`.
///
/// **Usage**: Apply this function immediately when reading package names from manifests
/// (e.g., manifest.package.name) to ensure all internal tracking uses snake_case consistently.
pub fn normalize_package_to_import_name(package_name: &str) -> String {
    package_name.replace('-', "_")
}
