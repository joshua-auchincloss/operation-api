use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
};

use operation_api_fs::{FileSystem, physical::Physical};
use operation_api_manifests::{
    config::NewForNamed,
    package::{Dependency, PackageManifest},
    version::Version,
};

use super::utils::normalize_import_to_package_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyMutability {
    Immutable,
    Mutable,
}

#[derive(Clone)]
pub struct ResolvedDependency {
    pub path: PathBuf,
    pub mutability: DependencyMutability,
    pub version: Version,
}

pub struct PackageResolver {
    pub(crate) fs: Arc<dyn FileSystem>,
}

impl PackageResolver {
    pub fn new() -> Self {
        Self {
            fs: Arc::new(Physical),
        }
    }

    pub fn with_fs(fs: Arc<dyn FileSystem>) -> Self {
        Self { fs }
    }

    pub fn resolve(
        &self,
        root_path: &Path,
        manifest: &PackageManifest,
        dep_name: &str,
    ) -> crate::Result<ResolvedDependency> {
        let package_name = normalize_import_to_package_name(dep_name);

        let dep = manifest
            .dependencies
            .get(&package_name)
            .ok_or_else(|| {
                crate::Error::UnresolvedDependency {
                    name: format!("{} (normalized from import '{}')", package_name, dep_name),
                }
            })?;

        match dep {
            Dependency::Path { path } => {
                let resolved_path = root_path.join(path);

                // Load the dependency's manifest to get its version
                let dep_manifest = PackageManifest::new(self.fs.as_ref(), &resolved_path)
                    .map_err(crate::Error::ManifestError)?;

                let version = Version::parse(&dep_manifest.package.version.to_string())?;

                Ok(ResolvedDependency {
                    path: resolved_path,
                    mutability: DependencyMutability::Mutable,
                    version,
                })
            },
            Dependency::Git { .. } | Dependency::Remote { .. } => {
                // TODO: Implement git and remote dependency resolution
                Err(crate::Error::UnresolvedDependency {
                    name: format!("{} (git/remote dependencies not yet supported)", dep_name),
                })
            },
        }
    }

    /// Compute a deterministic content hash for mutable dependencies.
    ///
    /// This hash is used to detect changes in local path dependencies and invalidate
    /// the compilation cache when source files are modified.
    ///
    /// # Determinism Guarantees
    ///
    /// The hash is deterministic within the bounds of Rust's `DefaultHasher`:
    /// - Files are sorted lexicographically before hashing
    /// - Both file path and content are included in the hash
    /// - File path is hashed first, then content
    /// - Hash output is stable within the same process
    ///
    /// **Note**: `DefaultHasher` is *not* guaranteed to be stable across different
    /// Rust versions or process invocations. For cross-process stability, consider
    /// using a cryptographic hash like SHA-256.
    ///
    /// # Implementation Details
    ///
    /// - Globs all `*.pld` files in the `schema/` directory
    /// - Respects package exclusion patterns
    /// - Includes file paths to avoid collisions from identical content in different files
    pub async fn compute_content_hash(
        &self,
        path: &Path,
    ) -> crate::Result<String> {
        let schema_dir = path.join("schema");
        let include = vec![format!("{}/**/*.pld", schema_dir.display())];
        let exclude: Vec<String> = vec![];

        let files = self.fs.find_glob(&include, &exclude)?;

        let mut hasher = DefaultHasher::new();

        // Sort files lexicographically for deterministic hashing
        let mut sorted_files = files;
        sorted_files.sort();

        // Hash both file path and content to avoid collisions
        for file in sorted_files {
            let content = self.fs.read_to_string(&file).await?;
            // Hash file path first for better distribution
            file.to_string_lossy().hash(&mut hasher);
            // Then hash content
            content.hash(&mut hasher);
        }

        Ok(format!("{:x}", hasher.finish()))
    }
}

impl Default for PackageResolver {
    fn default() -> Self {
        Self::new()
    }
}
