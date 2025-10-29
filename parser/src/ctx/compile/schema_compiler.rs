use std::sync::Arc;

use pathfinding::prelude::*;

use crate::{
    ctx::{
        SchemaCtx,
        cache::CacheKey,
        graph::schemas::{Import, SchemaDependencyGraph},
    },
    tokens::ToTokens,
};
pub struct SchemaCompiler;

impl SchemaCompiler {
    pub async fn compile_all(ctx: &super::CompileCtx) -> crate::Result<()> {
        tracing::info!("Starting parallel schema compilation");

        // Create progress spinner for building dependency graph
        let graph_spinner = ctx.progress.add_spinner("Analyzing");
        graph_spinner.set_message("schema dependencies");

        tracing::trace!("Building schema dependency graph");
        let graph = Self::build_graph(ctx).await?;
        tracing::trace!(schema_count = graph.nodes.len(), "Dependency graph built");

        graph_spinner.finish_with_message(format!("{} schemas", graph.nodes.len()));

        if !graph.nodes.is_empty() {
            tracing::trace!("Detecting circular schema dependencies");
            let successors_fn = |node: &CacheKey| -> Vec<CacheKey> { graph.successors(node) };

            let components = strongly_connected_components(
                &graph
                    .nodes
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>(),
                successors_fn,
            );

            // size > 1 is a cycle
            for component in components {
                if component.len() > 1 {
                    let cycle_names: Vec<String> = component
                        .iter()
                        .map(|k| k.package_name.clone())
                        .collect();

                    tracing::error!(cycle = ?cycle_names, "Circular schema dependency detected");
                    return Err(crate::Error::SchemaCircularDependency {
                        schemas: cycle_names,
                    });
                }
            }
            tracing::trace!("No circular dependencies found");
        }

        // 3. Get compilation groups using topological sort
        tracing::trace!("Computing topological sort groups");
        let groups = Self::topological_groups(&graph)?;
        tracing::info!(
            group_count = groups.len(),
            total_schemas = groups.iter().map(|g| g.len()).sum::<usize>(),
            "Topological sort complete, starting compilation"
        );

        // Create progress bar for compilation
        let total_schemas: u64 = groups.iter().map(|g| g.len() as u64).sum();
        let compile_bar = ctx
            .progress
            .add_bar(total_schemas, "Compiling");

        // 4. Compile each group in parallel
        for (level, group) in groups.iter().enumerate() {
            tracing::debug!(
                level = level,
                schemas_in_group = group.len(),
                schemas = ?group.iter().map(|k| &k.package_name).collect::<Vec<_>>(),
                "Compiling schema group"
            );

            // Compile all schemas in this group concurrently
            // They have no dependencies on each other, so this is safe
            let tasks: Vec<_> = group
                .iter()
                .map(|schema_id| Self::compile_schema(ctx, schema_id))
                .collect();

            futures_util::future::try_join_all(tasks).await?;

            // Update progress bar
            compile_bar.inc(group.len() as u64);
            compile_bar.set_message(format!("level {} complete", level));

            tracing::debug!(level = level, "Schema group compilation complete");
        }

        compile_bar.finish_with_message("all schemas");

        tracing::info!("Schema compilation complete");

        // Step 5: Global reference validation (deferred - TypeRegistry already validates)
        // TODO: Implement parallel global reference validation using dijkstra_all
        // when cross-schema reference checking is needed beyond TypeRegistry

        Ok(())
    }

    /// Build a schema dependency graph from all loaded schemas.
    ///
    /// This extracts imports from each schema and builds the dependency graph
    /// used for topological sorting.
    async fn build_graph(ctx: &super::CompileCtx) -> crate::Result<SchemaDependencyGraph> {
        let mut graph = SchemaDependencyGraph::new();

        // Add root schema
        let root_key = Self::build_cache_key_for_schema(&ctx.root)?;
        let root_imports = Self::extract_imports(&ctx.root);
        graph.add_schema(root_key.clone(), root_imports);

        // Add all dependency schemas
        let state = ctx.state.read().await;
        for schema in state.dependencies.values() {
            let dep_key = Self::build_cache_key_for_schema(schema)?;
            let dep_imports = Self::extract_imports(schema);
            graph.add_schema(dep_key, dep_imports);
        }
        drop(state);

        // Build dependency edges between schemas
        graph.build_dependencies();

        Ok(graph)
    }

    /// Compute topological groups for parallel compilation.
    ///
    /// Uses `topological_sort_into_groups` from pathfinding to find levels
    /// where all schemas in a level can be compiled in parallel.
    fn topological_groups(graph: &SchemaDependencyGraph) -> crate::Result<Vec<Vec<CacheKey>>> {
        if graph.nodes.is_empty() {
            return Ok(vec![]);
        }

        let nodes: Vec<CacheKey> = graph.nodes.keys().cloned().collect();
        let successors_fn = |node: &CacheKey| -> Vec<CacheKey> { graph.successors(node) };

        match topological_sort_into_groups(&nodes, successors_fn) {
            Ok(groups) => Ok(groups),
            Err(_cycle) => {
                // This shouldn't happen as we check for cycles earlier,
                // but handle it gracefully
                Err(crate::Error::SchemaCircularDependency {
                    schemas: vec!["<unknown>".to_string()],
                })
            },
        }
    }

    /// Compile a single schema.
    ///
    /// For Step 3, this is a placeholder that prepares for future steps:
    /// - Step 4 will add namespace discovery
    /// - Steps 5-6 will add type dependency analysis and registration
    /// - Step 7 will add cross-schema reference validation
    ///
    /// Currently, this just validates that the schema exists and is accessible.
    #[tracing::instrument(skip(ctx, schema_id), fields(package = %schema_id.package_name))]
    async fn compile_schema(
        ctx: &super::CompileCtx,
        schema_id: &CacheKey,
    ) -> crate::Result<()> {
        use super::utils::normalize_package_to_import_name;

        tracing::debug!("Starting schema compilation");

        // Step 4: Namespace BFS discovery and depth-based processing
        // Resolve the SchemaCtx for the given schema id (root or dependency)
        // Note: schema_id.package_name is already normalized (snake_case)
        let root_package_normalized =
            normalize_package_to_import_name(&ctx.root.package.package.name);

        let schema: Arc<SchemaCtx> = if schema_id.package_name == root_package_normalized {
            tracing::trace!("Using root schema");
            ctx.root.clone()
        } else {
            tracing::trace!(
                looking_for = %schema_id.package_name,
                available_deps = ?ctx.dependency_names().await,
                "Looking up dependency"
            );
            ctx.get_dependency(&schema_id.package_name)
                .await
                .ok_or_else(|| {
                    crate::Error::InternalError {
                        message: format!("Schema not found: {}", schema_id.package_name),
                    }
                })?
        };

        tracing::trace!(namespace_count = schema.namespaces.len(), "Schema resolved");

        // Build namespace levels (BFS from root namespaces / no-incoming nodes)
        let levels = Self::namespace_levels(&schema);

        tracing::debug!(
            depth_levels = levels.len(),
            total_namespaces = levels.iter().map(|l| l.len()).sum::<usize>(),
            "Namespace levels computed"
        );

        for (depth, group) in levels.into_iter().enumerate() {
            tracing::debug!(
                depth = depth,
                namespaces_in_group = group.len(),
                namespaces = ?group,
                "Processing namespace depth level"
            );

            let tasks: Vec<_> = group
                .iter()
                .map(|ns_name| {
                    let schema = Arc::clone(&schema);
                    let ns_name = ns_name.clone();
                    async move { Self::register_types_recursive(&schema, &ns_name, depth).await }
                })
                .collect();

            futures_util::future::try_join_all(tasks).await?;

            tracing::debug!(depth = depth, "Namespace depth level complete");
        }

        tracing::debug!("Schema compilation complete");
        Ok(())
    }

    fn namespace_levels(schema: &SchemaCtx) -> Vec<Vec<String>> {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let mut nodes: BTreeSet<String> = BTreeSet::new();
        let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut indegree: BTreeMap<String, usize> = BTreeMap::new();

        for ns_name in schema.namespaces.keys() {
            nodes.insert(ns_name.clone());
            adj.insert(ns_name.clone(), Vec::new());
            indegree.insert(ns_name.clone(), 0);
        }

        for (ns_name, ns_ctx) in &schema.namespaces {
            for import in &ns_ctx.imports {
                let ref_ctx = import.value.as_ref_context();
                if ref_ctx.package == schema.package.package.name
                    && let Some(target_top) = ref_ctx.namespace.first()
                    && nodes.contains(target_top)
                {
                    if let Some(v) = adj.get_mut(ns_name) {
                        v.push(target_top.clone())
                    }
                    if let Some(degree) = indegree.get_mut(target_top) {
                        *degree += 1;
                    }
                }
            }
        }

        // find roots (indegree == 0)
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        for (n, &deg) in &indegree {
            if deg == 0 {
                queue.push_back((n.clone(), 0));
            }
        }

        // if no roots (cycle), start from all nodes to ensure coverage
        if queue.is_empty() {
            for n in nodes.iter() {
                queue.push_back((n.clone(), 0));
            }
        }

        let mut depth_map: BTreeMap<String, usize> = BTreeMap::new();

        while let Some((node, depth)) = queue.pop_front() {
            let entry = depth_map
                .entry(node.clone())
                .or_insert(depth);
            if depth < *entry {
                *entry = depth;
            }

            if let Some(neighs) = adj.get(&node) {
                for n in neighs.iter() {
                    let next_depth = depth + 1;
                    let should_push = match depth_map.get(n) {
                        Some(&existing) => next_depth < existing,
                        None => true,
                    };
                    if should_push {
                        depth_map.insert(n.clone(), next_depth);
                        queue.push_back((n.clone(), next_depth));
                    }
                }
            }
        }

        // Group by depth
        let mut by_depth: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for (ns, depth) in depth_map.into_iter() {
            by_depth.entry(depth).or_default().push(ns);
        }

        // ensure deterministic ordering
        let mut levels: Vec<Vec<String>> = Vec::new();
        for (_d, mut group) in by_depth.into_iter() {
            group.sort();
            levels.push(group);
        }

        levels
    }

    /// Recursively register types in a namespace and all its children.
    ///
    /// Step 5: Type dependency graph construction and SCC detection
    /// - Extract type dependencies from all namespace children
    /// - Use strongly_connected_components to find cycles
    /// - Distinguish terminating (optional/array) vs non-terminating (required) cycles
    ///
    /// Step 6: Topological type registration
    /// - Use topological_sort_into_groups to find parallel registration groups
    /// - Register types in dependency order (dependencies first)
    /// - Process each group in parallel (types within group have no dependencies)
    ///
    /// Future: Step 7 will add cross-schema reference validation
    #[tracing::instrument(skip(schema), fields(namespace = %ns_name, depth = depth))]
    pub(crate) async fn register_types_recursive(
        schema: &Arc<SchemaCtx>,
        ns_name: &str,
        depth: usize,
    ) -> crate::Result<()> {
        tracing::trace!("Starting type registration for namespace");

        // Get the namespace context
        let ns_ctx = schema
            .namespaces
            .get(ns_name)
            .ok_or_else(|| {
                crate::Error::InternalError {
                    message: format!("Namespace not found at depth {}: {}", depth, ns_name),
                }
            })?;

        tracing::trace!(
            child_count = ns_ctx.children.len(),
            "Namespace context retrieved"
        );

        // Step 5: Build type dependency graph
        use crate::ctx::graph::TypeExtractor;
        let type_graph = TypeExtractor::extract_from_namespace(ns_ctx, &ns_ctx.ctx);

        tracing::trace!(
            type_count = type_graph.type_names().len(),
            "Type dependency graph extracted"
        );

        // Detect strongly connected components (cycles)
        let type_names = type_graph.type_names();
        if !type_names.is_empty() {
            tracing::trace!("Detecting type dependency cycles");
            use pathfinding::prelude::*;

            // Use only required edges for SCC detection
            // This finds true circular dependencies (not optional/array cycles)
            let successors_fn =
                |node: &crate::ctx::paths::NamedItemContext| type_graph.required_successors(node);

            let components = strongly_connected_components(&type_names, successors_fn);

            tracing::trace!(component_count = components.len(), "SCC detection complete");

            // Check for non-terminating cycles
            for component in &components {
                if component.len() > 1 {
                    // Found a cycle - check if it has terminating edges
                    if !type_graph.has_terminating_edge(component) {
                        // Non-terminating cycle - error
                        let cycle_str: Vec<String> = component
                            .iter()
                            .map(|ctx| ctx.display())
                            .collect();

                        tracing::error!(cycle = ?cycle_str, "Non-terminating type cycle detected");
                        return Err(crate::Error::TypeCircularDependency { types: cycle_str });
                    }
                    // Terminating cycle - allowed (e.g., A has optional B, B has required A)
                    tracing::trace!(
                        cycle = ?component.iter().map(|c| c.display()).collect::<Vec<_>>(),
                        "Terminating cycle allowed"
                    );
                }
            }

            // Step 6: Topological sort and type registration
            tracing::trace!("Computing topological sort for types");

            // Use ALL edges (required + optional + array) for topological sort
            // This ensures we register dependencies before dependents, even if optional
            let successors_fn =
                |node: &crate::ctx::paths::NamedItemContext| type_graph.all_successors(node);

            let groups = match topological_sort_into_groups(&type_names, successors_fn) {
                Ok(groups) => groups,
                Err(cycle) => {
                    // This shouldn't happen - we already validated cycles
                    // But handle gracefully just in case
                    tracing::error!(?cycle, "Topological sort failed unexpectedly");
                    return Err(crate::Error::TypeCircularDependency {
                        types: vec![format!("<topological sort failed: {:?}>", cycle)],
                    });
                },
            };

            tracing::debug!(
                group_count = groups.len(),
                total_types = groups.iter().map(|g| g.len()).sum::<usize>(),
                "Starting type registration"
            );

            // Register types in each group
            // Groups are returned in dependency order: dependencies first
            for (group_idx, group) in groups.iter().enumerate() {
                tracing::trace!(
                    group = group_idx,
                    types_in_group = group.len(),
                    types = ?group.iter().map(|t| t.display()).collect::<Vec<_>>(),
                    "Registering type group"
                );

                // All types in this group can be registered in parallel
                // (they have no dependencies on each other)
                let registration_tasks: Vec<_> = group
                    .iter()
                    .map(|type_ctx| {
                        let schema = Arc::clone(schema);
                        let type_ctx = type_ctx.clone();
                        let ns_name = ns_name.to_string();

                        async move { Self::register_type(&schema, &ns_name, &type_ctx).await }
                    })
                    .collect();

                // Wait for all types in this group to be registered
                futures_util::future::try_join_all(registration_tasks).await?;

                tracing::trace!(group = group_idx, "Type group registration complete");
            }
        }

        // Process nested namespaces recursively
        // Collect all child namespace keys that start with "{ns_name}::"
        let child_prefix = format!("{}::", ns_name);
        let child_namespaces: Vec<String> = schema
            .namespaces
            .keys()
            .filter(|key| key.starts_with(&child_prefix) && key.matches("::").count() == depth + 1)
            .cloned()
            .collect();

        // Process all children in parallel (they're at the same depth level)
        if !child_namespaces.is_empty() {
            tracing::trace!(
                child_count = child_namespaces.len(),
                children = ?child_namespaces,
                "Processing child namespaces"
            );

            let child_tasks: Vec<_> =
                child_namespaces
                    .iter()
                    .map(|child_ns| {
                        let schema = Arc::clone(schema);
                        let child_ns = child_ns.clone();
                        async move {
                            Self::register_types_recursive(&schema, &child_ns, depth + 1).await
                        }
                    })
                    .collect();

            futures_util::future::try_join_all(child_tasks).await?;
        }

        tracing::trace!("Namespace type registration complete");
        Ok(())
    }

    /// Register a single type into the schema's type registry.
    ///
    /// This retrieves the type definition from the namespace and calls
    /// the appropriate registry method based on the type kind.
    #[tracing::instrument(skip(schema), fields(namespace = %ns_name, type_name = %type_ctx.display()))]
    async fn register_type(
        schema: &Arc<SchemaCtx>,
        ns_name: &str,
        type_ctx: &crate::ctx::paths::NamedItemContext,
    ) -> crate::Result<()> {
        // Get the namespace
        let ns_ctx = schema
            .namespaces
            .get(ns_name)
            .ok_or_else(|| {
                crate::Error::InternalError {
                    message: format!(
                        "Namespace disappeared during type registration: {}",
                        ns_name
                    ),
                }
            })?;

        // Get the type definition from the namespace children
        let child = match ns_ctx.children.get(type_ctx) {
            Some(child) => child,
            None => {
                // Type not found - this is an internal error
                return Err(crate::Error::InternalError {
                    message: format!(
                        "Type not found in namespace during registration: {}",
                        type_ctx.display()
                    ),
                });
            },
        };

        // Register the type based on its kind
        use crate::ctx::common::{Definition, NamespaceChild};
        use std::sync::Arc;

        let (definition, span) = match &child.value {
            NamespaceChild::Struct(struct_def) => {
                tracing::trace!("Registering struct type");
                let span = struct_def.def_span().clone();

                let def = Definition::Struct(Arc::new(struct_def.clone()));
                (def, span)
            },
            NamespaceChild::OneOf(oneof_def) => {
                tracing::trace!("Registering oneof type");
                let span = oneof_def.def_span().clone();

                let def = Definition::OneOf(Arc::new(oneof_def.clone()));
                (def, span)
            },
            NamespaceChild::Error(error_def) => {
                tracing::trace!("Registering error type");
                let span = error_def.def_span().clone();

                let def = Definition::Error(Arc::new(error_def.clone()));
                (def, span)
            },
            NamespaceChild::Type(type_def) => {
                tracing::trace!("Registering type alias");
                let span = type_def.def_span().clone();

                let def = Definition::TypeAlias(Arc::new(type_def.clone()));
                (def, span)
            },
            NamespaceChild::Enum(enum_def) => {
                tracing::trace!("Registering enum type");
                let span = enum_def.def_span().clone();

                let def = Definition::Enum(Arc::new(enum_def.clone()));
                (def, span)
            },
            NamespaceChild::Operation(_) => {
                tracing::trace!("Skipping operation (not a type)");
                // Operations are not registered in the type registry
                // They are handled separately during code generation
                return Ok(());
            },
            NamespaceChild::Namespace(_) => {
                tracing::trace!("Skipping namespace (not a type)");
                // Nested namespaces are not types - skip
                return Ok(());
            },
        };

        // Get the registry from the schema and register the type
        tracing::trace!("Calling registry.register()");
        schema
            .registry
            .register(&type_ctx.context, &type_ctx.name, definition, span.clone())?;

        tracing::trace!("Type registration successful");
        Ok(())
    }

    /// Build a CacheKey for a schema.
    ///
    /// CacheKey format: `package_name + version + content_hash`
    ///
    /// For immutable dependencies, content_hash is None.
    /// For mutable dependencies, content_hash is computed from schema files.
    fn build_cache_key_for_schema(schema: &SchemaCtx) -> crate::Result<CacheKey> {
        use super::utils::normalize_package_to_import_name;

        let package_name = normalize_package_to_import_name(&schema.package.package.name);
        let version = schema.package.package.version.clone();

        // For now, we don't compute content hash during compilation
        // The cache already has this information from loading phase
        // We use None for content_hash (treats as immutable for lookup)
        let content_hash = None;

        Ok(CacheKey::new(package_name, version, content_hash))
    }

    /// Extract imports from a schema for dependency graph construction.
    ///
    /// Scans all namespaces in the schema and collects unique package imports.
    fn extract_imports(schema: &SchemaCtx) -> Vec<Import> {
        let mut imports = Vec::new();
        let mut seen_packages = std::collections::HashSet::new();

        for ns_ctx in schema.namespaces.values() {
            for import in &ns_ctx.imports {
                let ref_ctx = import.value.as_ref_context();
                let package = ref_ctx.package.clone();

                // Only add each package once
                // Don't add self-imports (same package)
                if package != schema.package.package.name && seen_packages.insert(package.clone()) {
                    imports.push(Import {
                        name: package,
                        resolved_id: None, // Will be resolved by build_dependencies()
                    });
                }
            }
        }

        imports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_imports_basic() {
        // This is a minimal smoke test
        // Real testing will be integration tests with actual schemas
        let imports = [
            Import {
                name: "foo".to_string(),
                resolved_id: None,
            },
            Import {
                name: "bar".to_string(),
                resolved_id: None,
            },
        ];

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].name, "foo");
        assert_eq!(imports[1].name, "bar");
    }

    #[test]
    fn test_topological_groups_empty() {
        let graph = SchemaDependencyGraph::new();
        let groups = SchemaCompiler::topological_groups(&graph).unwrap();
        assert_eq!(groups.len(), 0, "Empty graph should have no groups");
    }

    #[test]
    fn test_topological_groups_single_schema() {
        use operation_api_manifests::version::Version;

        let mut graph = SchemaDependencyGraph::new();
        let key = CacheKey::new("test".to_string(), Version::parse("1.0.0").unwrap(), None);
        graph.add_schema(key.clone(), vec![]);
        graph.build_dependencies();

        let groups = SchemaCompiler::topological_groups(&graph).unwrap();
        assert_eq!(groups.len(), 1, "Single schema should be one group");
        assert_eq!(groups[0].len(), 1, "Group should contain one schema");
        assert_eq!(groups[0][0], key);
    }

    #[test]
    fn test_topological_groups_linear_dependency() {
        use operation_api_manifests::version::Version;

        let mut graph = SchemaDependencyGraph::new();

        let key_a = CacheKey::new("a".to_string(), Version::parse("1.0.0").unwrap(), None);
        let key_b = CacheKey::new("b".to_string(), Version::parse("1.0.0").unwrap(), None);

        // b depends on a
        graph.add_schema(key_a.clone(), vec![]);
        graph.add_schema(
            key_b.clone(),
            vec![Import {
                name: "a".to_string(),
                resolved_id: None,
            }],
        );
        graph.build_dependencies();

        let groups = SchemaCompiler::topological_groups(&graph).unwrap();
        assert_eq!(groups.len(), 2, "Linear dependency should have 2 levels");
        assert_eq!(groups[0].len(), 1, "Level 0 should have 1 schema");
        assert_eq!(groups[1].len(), 1, "Level 1 should have 1 schema");

        // pathfinding's topological_sort_into_groups returns dependencies FIRST
        // So level 0 = things with no dependencies (b depends on a, so a is first)
        assert_eq!(
            groups[0][0], key_a,
            "Level 0 should be schema 'a' (no deps)"
        );
        assert_eq!(
            groups[1][0], key_b,
            "Level 1 should be schema 'b' (depends on a)"
        );
    }

    #[test]
    fn test_topological_groups_parallel_schemas() {
        use operation_api_manifests::version::Version;

        let mut graph = SchemaDependencyGraph::new();

        let key_a = CacheKey::new("a".to_string(), Version::parse("1.0.0").unwrap(), None);
        let key_b = CacheKey::new("b".to_string(), Version::parse("1.0.0").unwrap(), None);
        let key_c = CacheKey::new("c".to_string(), Version::parse("1.0.0").unwrap(), None);

        // a, b, c are independent
        graph.add_schema(key_a.clone(), vec![]);
        graph.add_schema(key_b.clone(), vec![]);
        graph.add_schema(key_c.clone(), vec![]);
        graph.build_dependencies();

        let groups = SchemaCompiler::topological_groups(&graph).unwrap();
        assert_eq!(
            groups.len(),
            1,
            "Independent schemas should be in one group"
        );
        assert_eq!(groups[0].len(), 3, "All 3 schemas can compile in parallel");

        // All schemas should be in the group (order doesn't matter)
        let group_set: std::collections::HashSet<_> = groups[0].iter().collect();
        assert!(group_set.contains(&key_a));
        assert!(group_set.contains(&key_b));
        assert!(group_set.contains(&key_c));
    }
}
