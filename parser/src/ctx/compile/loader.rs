use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_channel::{Receiver, Sender};
use operation_api_fs::FileSystem;
use operation_api_manifests::{
    lock::LockedSource,
    package::{Dependency, PackageManifest},
    version::Version,
};
use tokio::sync::RwLock;

use crate::ctx::{
    SchemaCtx,
    cache::{CacheKey, CachedSchema, SchemaCache},
    registry::TypeRegistry,
};

use super::{
    progress::CompilationProgress,
    resolver::{DependencyMutability, PackageResolver, ResolvedDependency},
    state::{ResolvedMetadata, SharedCompilationState},
    utils::{normalize_import_to_package_name, normalize_package_to_import_name},
};

#[derive(Clone)]
pub(super) struct CompilationTask {
    pub package_name: String,
    pub dependency_chain: Vec<String>,
}

pub(super) enum DependencyTaskResult {
    Loaded(DependencyResult),
    AlreadyLoaded,
}

pub(super) struct DependencyResult {
    pub package_name: String,
    pub schema: Arc<SchemaCtx>,
    pub version: Version,
}

pub struct DependencyLoader;

impl DependencyLoader {
    pub async fn load_dependencies_parallel(
        root: &SchemaCtx,
        state: Arc<RwLock<SharedCompilationState>>,
        resolver: Arc<PackageResolver>,
        cache: SchemaCache,
        type_registry: TypeRegistry,
        root_path: PathBuf,
        max_concurrent_tasks: usize,
        progress: &CompilationProgress,
    ) -> crate::Result<()> {
        let (task_tx, task_rx): (Sender<CompilationTask>, Receiver<CompilationTask>) =
            async_channel::unbounded();
        let (result_tx, result_rx): (
            Sender<Result<DependencyTaskResult, crate::Error>>,
            Receiver<Result<DependencyTaskResult, crate::Error>>,
        ) = async_channel::unbounded();

        let resolving_spinner = progress.add_spinner("Resolving");
        resolving_spinner.set_message("dependencies");

        let mut seen_packages = HashSet::new();
        let mut initial_tasks = Vec::new();
        for ns_ctx in root.namespaces.values() {
            for import in &ns_ctx.imports {
                let pkg = import.value.as_ref_context().package.clone();
                if !root.namespaces.contains_key(&pkg) && seen_packages.insert(pkg.to_string()) {
                    initial_tasks.push(CompilationTask {
                        package_name: pkg.to_string(),
                        dependency_chain: vec![normalize_package_to_import_name(
                            &root.package.package.name,
                        )],
                    });
                }
            }
        }

        for task in initial_tasks {
            if task_tx.send(task).await.is_err() {
                return Err(crate::Error::InternalError {
                    message: "Failed to send initial compilation task".into(),
                });
            }
        }

        drop(task_tx);

        let mut workers = Vec::new();
        for worker_id in 0..max_concurrent_tasks.max(1) {
            let task_rx = task_rx.clone();
            let result_tx = result_tx.clone();

            let state = state.clone();
            let resolver = resolver.clone();
            let cache = cache.clone();
            let root_path = root_path.clone();
            let root_package = Arc::new(root.package.clone());
            let registry = type_registry.clone();

            let worker = Box::pin(async move {
                while let Ok(task) = task_rx.recv().await {
                    tracing::debug!("Worker {} processing {}", worker_id, task.package_name);

                    match Self::process_dependency_task(
                        task,
                        state.clone(),
                        resolver.clone(),
                        cache.clone(),
                        root_path.clone(),
                        root_package.clone(),
                        registry.clone(),
                    )
                    .await
                    {
                        Err(e) => {
                            let _ = result_tx.send(Err(e)).await;
                            break;
                        },
                        Ok(task_result) => {
                            if result_tx
                                .send(Ok(task_result))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        },
                    }
                }
            });

            workers.push(tokio::spawn(worker));
        }

        let result_collector = tokio::spawn({
            let state = state.clone();
            let resolving_spinner = resolving_spinner.clone();

            async move {
                let mut tasks_remaining = 0usize;
                let mut first_error: Option<crate::Error> = None;
                let mut loaded_count = 0usize;

                while let Ok(result_or_error) = result_rx.recv().await {
                    match result_or_error {
                        Ok(task_result) => {
                            match task_result {
                                DependencyTaskResult::Loaded(result) => {
                                    loaded_count += 1;
                                    resolving_spinner.set_message(format!(
                                        "{}@{} ({} loaded)",
                                        result.package_name, result.version, loaded_count
                                    ));

                                    let mut state = state.write().await;

                                    state
                                        .dependencies
                                        .insert(result.package_name.clone(), result.schema.clone());
                                    state
                                        .loaded_versions
                                        .insert(result.package_name.clone(), result.version);
                                    state
                                        .processing_set
                                        .remove(&result.package_name);

                                    tasks_remaining = tasks_remaining.saturating_sub(1);
                                },
                                DependencyTaskResult::AlreadyLoaded => {
                                    tasks_remaining = tasks_remaining.saturating_sub(1);
                                },
                            }
                        },
                        Err(e) => {
                            if first_error.is_none() {
                                first_error = Some(e);
                            }
                        },
                    }
                }

                resolving_spinner.finish_with_message(format!("{} dependencies", loaded_count));

                if let Some(err) = first_error {
                    Err(err)
                } else {
                    Ok(())
                }
            }
        });

        drop(result_tx);

        let collector_result = result_collector.await;

        for f in futures_util::future::join_all(workers).await {
            if let Err(e) = f {
                return Err(crate::Error::InternalError {
                    message: format!("Worker task failed: {}", e),
                });
            }
        }

        match collector_result {
            Ok(Err(e)) => Err(e),
            Err(e) => panic!("{e:#?}"),
            Ok(Ok(_)) => Ok(()),
        }
    }

    async fn process_dependency_task(
        task: CompilationTask,
        state: Arc<RwLock<SharedCompilationState>>,
        resolver: Arc<PackageResolver>,
        cache: SchemaCache,
        root_path: PathBuf,
        root_package: Arc<PackageManifest>,
        registry: TypeRegistry,
    ) -> crate::Result<DependencyTaskResult> {
        let dep_name = &task.package_name;

        {
            let state_read = state.read().await;
            if state_read.processing_set.contains(dep_name) {
                let mut chain = task.dependency_chain.clone();
                chain.push(dep_name.clone());
                return Err(crate::Error::CircularDependency { chain });
            }

            if state_read
                .loaded_versions
                .contains_key(dep_name)
            {
                return Ok(DependencyTaskResult::AlreadyLoaded);
            }
        }

        {
            let mut state_write = state.write().await;
            state_write
                .processing_set
                .insert(dep_name.clone());
        }

        let resolved = resolver.resolve(&root_path, root_package.as_ref(), dep_name)?;

        let use_version = Self::resolve_version(&state, dep_name, &resolved).await?;

        let checksum = resolver
            .compute_content_hash(&resolved.path)
            .await?;

        Self::validate_checksum(&state, dep_name, &use_version, &checksum).await;

        let cache_key = Self::build_cache_key(
            dep_name,
            &use_version,
            &resolved.path,
            resolved.mutability,
            &resolver,
        )
        .await?;

        let dep_schema = Self::load_or_cache_schema(
            &cache,
            &cache_key,
            &use_version,
            &resolved,
            registry.clone(),
            &resolver,
        )
        .await?;

        if let Some(dep_lockfile) =
            Self::load_dependency_lockfile(resolver.fs.as_ref(), &resolved.path).await
        {
            Self::merge_dependency_lockfile(&state, dep_lockfile).await;
        }

        let source = Self::build_locked_source(&root_package, dep_name, &resolved.path);

        let mut provides = BTreeSet::new();
        for ns_name in dep_schema.namespaces.keys() {
            provides.insert(ns_name.clone());
        }

        let (_transitive_deps, dependency_names) =
            Self::collect_transitive_deps(&dep_schema, &state, &task.dependency_chain, dep_name)
                .await;

        {
            let mut state_write = state.write().await;
            state_write.resolved_metadata.insert(
                dep_name.clone(),
                ResolvedMetadata {
                    version: use_version.clone(),
                    source,
                    checksum,
                    provides,
                    dependencies: dependency_names,
                },
            );
        }

        Ok(DependencyTaskResult::Loaded(DependencyResult {
            package_name: dep_name.clone(),
            schema: dep_schema,
            version: use_version,
        }))
    }

    async fn resolve_version(
        state: &Arc<RwLock<SharedCompilationState>>,
        dep_name: &str,
        resolved: &ResolvedDependency,
    ) -> crate::Result<Version> {
        let state_read = state.read().await;
        let mut candidate_version = resolved.version.clone();

        if let Some(existing_version) = state_read.loaded_versions.get(dep_name) {
            if !existing_version.is_compatible(candidate_version.clone()) {
                return Err(crate::Error::VersionIncompatibility {
                    package: dep_name.to_string(),
                    required: candidate_version.to_string(),
                    found: existing_version.to_string(),
                });
            }
            if existing_version > &candidate_version {
                candidate_version = existing_version.clone();
            }
        }

        if let Some(lockfile) = &state_read.lockfile {
            let pkg_name_kebab = normalize_import_to_package_name(dep_name);
            for locked_pkg in lockfile.packages.values() {
                if locked_pkg.name == pkg_name_kebab {
                    let locked_version = &locked_pkg.version;
                    if locked_version.is_compatible(candidate_version.clone())
                        && locked_version > &candidate_version
                    {
                        candidate_version = locked_version.clone();
                    }
                }
            }
        }

        Ok(candidate_version)
    }

    async fn validate_checksum(
        state: &Arc<RwLock<SharedCompilationState>>,
        dep_name: &str,
        use_version: &Version,
        checksum: &str,
    ) {
        let should_invalidate = {
            let state_read = state.read().await;
            if let Some(lockfile) = &state_read.lockfile {
                let pkg_name_kebab = normalize_import_to_package_name(dep_name);
                let key = format!("{}@{}", pkg_name_kebab, use_version);
                if let Some(locked_pkg) = lockfile.packages.get(&key) {
                    if locked_pkg.checksum != checksum {
                        Some((locked_pkg.checksum.clone(), checksum.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some((expected, got)) = should_invalidate {
            let mut state_write = state.write().await;
            state_write.lockfile_invalidated = true;
            tracing::warn!(
                "checksum mismatch for package {}: expected '{}', got '{}'. lockfile will be regenerated.",
                dep_name,
                expected,
                got
            );
        }
    }

    async fn build_cache_key(
        package_name: &str,
        version: &Version,
        path: &Path,
        mutability: DependencyMutability,
        resolver: &PackageResolver,
    ) -> crate::Result<CacheKey> {
        let content_hash = match mutability {
            DependencyMutability::Immutable => None,
            DependencyMutability::Mutable => Some(resolver.compute_content_hash(path).await?),
        };

        Ok(CacheKey::new(
            package_name.to_string(),
            version.clone(),
            content_hash,
        ))
    }

    async fn load_or_cache_schema(
        cache: &SchemaCache,
        cache_key: &CacheKey,
        use_version: &Version,
        resolved: &ResolvedDependency,
        registry: TypeRegistry,
        resolver: &PackageResolver,
    ) -> crate::Result<Arc<SchemaCtx>> {
        if let Some(cached) = cache.get(cache_key).await {
            if !cached
                .version
                .is_compatible(use_version.clone())
            {
                Self::load_schema_fresh(resolver.fs.as_ref(), resolved, registry).await
            } else {
                Ok(cached.schema)
            }
        } else {
            let schema = Self::load_schema_fresh(resolver.fs.as_ref(), resolved, registry).await?;
            cache
                .insert(
                    cache_key.clone(),
                    CachedSchema::new(schema.clone(), use_version.clone()),
                )
                .await;
            Ok(schema)
        }
    }

    async fn load_schema_fresh(
        fs: &dyn FileSystem,
        resolved: &ResolvedDependency,
        registry: TypeRegistry,
    ) -> crate::Result<Arc<SchemaCtx>> {
        let schema = SchemaCtx::from_path(fs, &resolved.path, registry).await?;
        Ok(Arc::new(schema))
    }

    async fn load_dependency_lockfile(
        fs: &dyn FileSystem,
        dep_path: &Path,
    ) -> Option<operation_api_manifests::lock::Lockfile> {
        use operation_api_manifests::{config::NewForNamed, lock::Lockfiles};
        match Lockfiles::new_for_opt(fs, dep_path) {
            Ok(Some(Lockfiles::V1(lockfile))) => Some(lockfile),
            _ => None,
        }
    }

    async fn merge_dependency_lockfile(
        state: &Arc<RwLock<SharedCompilationState>>,
        dep_lockfile: operation_api_manifests::lock::Lockfile,
    ) {
        let mut state_write = state.write().await;

        for (key, locked_pkg) in dep_lockfile.packages {
            let pkg_name_snake = normalize_package_to_import_name(&locked_pkg.name);

            if let Some(existing_version) = state_write
                .loaded_versions
                .get(&pkg_name_snake)
                && existing_version >= &locked_pkg.version
            {
                continue;
            }

            if let Some(our_lockfile) = &mut state_write.lockfile {
                let should_add = our_lockfile
                    .packages
                    .get(&key)
                    .map(|existing| locked_pkg.version > existing.version)
                    .unwrap_or(true);

                if should_add {
                    our_lockfile.packages.insert(key, locked_pkg);
                }
            }
        }
    }

    fn build_locked_source(
        root_package: &PackageManifest,
        dep_name: &str,
        resolved_path: &Path,
    ) -> LockedSource {
        match &root_package
            .dependencies
            .get(&normalize_import_to_package_name(dep_name))
        {
            Some(Dependency::Path { path }) => LockedSource::Path { path: path.clone() },
            Some(Dependency::Git { git, git_ref, rev }) => {
                LockedSource::Git {
                    url: git.clone(),
                    git_ref: git_ref.clone(),
                    rev: rev.clone(),
                }
            },
            Some(Dependency::Remote { registry, .. }) => {
                LockedSource::Registry {
                    url: registry
                        .clone()
                        .unwrap_or_else(|| "https://tbd.com".to_string()),
                }
            },
            None => {
                LockedSource::Path {
                    path: resolved_path.to_path_buf(),
                }
            },
        }
    }

    async fn collect_transitive_deps(
        dep_schema: &Arc<SchemaCtx>,
        state: &Arc<RwLock<SharedCompilationState>>,
        dependency_chain: &[String],
        dep_name: &str,
    ) -> (Vec<CompilationTask>, Vec<String>) {
        let mut transitive_deps = Vec::new();
        let mut dependency_names = Vec::new();
        let mut new_chain = dependency_chain.to_vec();
        new_chain.push(dep_name.to_string());

        for ns_ctx in dep_schema.namespaces.values() {
            for nested_import in &ns_ctx.imports {
                let object = nested_import.value.as_ref_context();
                let state_read = state.read().await;
                if !dep_schema
                    .namespaces
                    .contains_key(&object.package)
                    && !state_read
                        .loaded_versions
                        .contains_key(&object.package)
                {
                    drop(state_read);

                    dependency_names.push(object.package.to_string());
                    transitive_deps.push(CompilationTask {
                        package_name: object.package.to_string(),
                        dependency_chain: new_chain.clone(),
                    });
                }
            }
        }

        (transitive_deps, dependency_names)
    }
}
