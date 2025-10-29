// Schema dependency graph for multi-schema compilation
//
// This module implements the schema dependency graph used for determining
// the correct order to compile schemas in a workspace. Schemas depend on
// each other through import statements.

use std::collections::{HashMap, HashSet};

use crate::ctx::cache::CacheKey;

/// An import statement (use statement in lib.pld)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Import {
    /// The imported package/module name
    pub name: String,

    /// Optional: resolved schema ID
    pub resolved_id: Option<CacheKey>,
}

/// A node in the schema dependency graph
#[derive(Debug, Clone)]
pub struct SchemaNode {
    /// Unique identifier for this schema
    pub id: CacheKey,

    /// Imports declared in this schema's lib.pld
    pub imports: Vec<Import>,

    /// Computed: schemas this schema depends on
    pub depends_on: Vec<CacheKey>,
}

/// Schema dependency graph for workspace compilation
///
/// This graph tracks dependencies between schemas in a workspace.
/// It's used with pathfinding's topological_sort_into_groups to:
/// 1. Detect circular dependencies between schemas
/// 2. Determine compilation order
/// 3. Enable parallel compilation of independent schemas
#[derive(Debug, Clone, Default)]
pub struct SchemaDependencyGraph {
    /// All schema nodes indexed by schema ID
    pub(crate) nodes: HashMap<CacheKey, SchemaNode>,
}

impl SchemaDependencyGraph {
    /// Create a new empty schema dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add a schema to the graph
    pub fn add_schema(
        &mut self,
        id: CacheKey,
        imports: Vec<Import>,
    ) {
        self.nodes.insert(
            id.clone(),
            SchemaNode {
                id,
                imports,
                depends_on: Vec::new(),
            },
        );
    }

    /// Build dependencies by resolving imports
    ///
    /// This resolves import names to schema IDs and populates
    /// the `depends_on` field for each schema.
    ///
    /// The resolution works by matching import names to package names
    /// in the CacheKey schema IDs.
    pub fn build_dependencies(&mut self) {
        // Create mapping from package names to schema IDs
        let mut name_to_id: HashMap<String, CacheKey> = HashMap::new();

        // Extract package name from CacheKey (it's a public field)
        for schema_id in self.nodes.keys() {
            name_to_id.insert(schema_id.package_name.clone(), schema_id.clone());
        }

        // Resolve imports
        let schema_ids: Vec<_> = self.nodes.keys().cloned().collect();

        for schema_id in schema_ids {
            let imports = self.nodes[&schema_id].imports.clone();
            let mut depends_on = Vec::new();

            for import in imports {
                // First check if import already has resolved_id
                if let Some(resolved_id) = import.resolved_id {
                    if resolved_id != schema_id {
                        depends_on.push(resolved_id);
                    }
                    continue;
                }

                // Try to find by package name
                if let Some(dep_id) = name_to_id.get(&import.name)
                    && dep_id != &schema_id
                {
                    depends_on.push(dep_id.clone());
                }
            }

            // Update the node with resolved dependencies
            if let Some(node) = self.nodes.get_mut(&schema_id) {
                node.depends_on = depends_on;
            }
        }
    }

    /// Get all schema IDs in the graph
    pub fn schema_ids(&self) -> Vec<CacheKey> {
        self.nodes.keys().cloned().collect()
    }

    /// Get successors for a schema (schemas that depend on this one)
    ///
    /// This is used by pathfinding's topological_sort_into_groups.
    /// Returns schemas that list this schema_id in their depends_on.
    ///
    /// For topological sort, if B depends on A, then A's successor is B.
    /// This ensures A is processed before B.
    pub fn successors(
        &self,
        schema_id: &CacheKey,
    ) -> Vec<CacheKey> {
        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.depends_on.contains(schema_id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the node for a schema
    pub fn get_node(
        &self,
        schema_id: &CacheKey,
    ) -> Option<&SchemaNode> {
        self.nodes.get(schema_id)
    }

    /// Check if a cycle exists in dependencies
    pub fn has_circular_dependency(
        &self,
        cycle: &[CacheKey],
    ) -> bool {
        // A cycle is circular if any schema in the cycle depends on another
        let cycle_set: HashSet<_> = cycle.iter().collect();

        for schema_id in cycle {
            if let Some(node) = self.nodes.get(schema_id) {
                for dep in &node.depends_on {
                    if cycle_set.contains(&dep) && dep != schema_id {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use operation_api_manifests::version::Version;

    fn schema_id(name: &str) -> CacheKey {
        CacheKey::new(name.to_string(), Version::parse("1.0.0").unwrap(), None)
    }

    fn import(name: &str) -> Import {
        Import {
            name: name.to_string(),
            resolved_id: None,
        }
    }

    fn import_resolved(
        name: &str,
        resolved: CacheKey,
    ) -> Import {
        Import {
            name: name.to_string(),
            resolved_id: Some(resolved),
        }
    }

    #[test]
    fn test_no_dependencies() {
        let mut graph = SchemaDependencyGraph::new();

        graph.add_schema(schema_id("schema_a"), vec![]);
        graph.add_schema(schema_id("schema_b"), vec![]);

        graph.build_dependencies();

        assert_eq!(
            graph
                .successors(&schema_id("schema_a"))
                .len(),
            0
        );
        assert_eq!(
            graph
                .successors(&schema_id("schema_b"))
                .len(),
            0
        );
    }

    #[test]
    fn test_linear_dependencies() {
        let mut graph = SchemaDependencyGraph::new();

        let schema_a_id = schema_id("schema_a");
        let schema_b_id = schema_id("schema_b");
        let schema_c_id = schema_id("schema_c");

        // A depends on B
        graph.add_schema(
            schema_a_id.clone(),
            vec![import_resolved("schema_b", schema_b_id.clone())],
        );
        // B depends on C
        graph.add_schema(
            schema_b_id.clone(),
            vec![import_resolved("schema_c", schema_c_id.clone())],
        );
        // C has no dependencies
        graph.add_schema(schema_c_id.clone(), vec![]);

        graph.build_dependencies();

        // successors() returns "who depends on me"
        // C is depended on by B
        let c_successors = graph.successors(&schema_c_id);
        assert_eq!(c_successors.len(), 1);
        assert!(c_successors.contains(&schema_b_id));

        // B is depended on by A
        let b_successors = graph.successors(&schema_b_id);
        assert_eq!(b_successors.len(), 1);
        assert!(b_successors.contains(&schema_a_id));

        // A has no dependents
        assert_eq!(graph.successors(&schema_a_id).len(), 0);
    }

    #[test]
    fn test_diamond_dependencies() {
        let mut graph = SchemaDependencyGraph::new();

        let schema_a_id = schema_id("schema_a");
        let schema_b_id = schema_id("schema_b");
        let schema_c_id = schema_id("schema_c");
        let schema_d_id = schema_id("schema_d");

        // D depends on B and C
        graph.add_schema(
            schema_d_id.clone(),
            vec![
                import_resolved("schema_b", schema_b_id.clone()),
                import_resolved("schema_c", schema_c_id.clone()),
            ],
        );
        // B depends on A
        graph.add_schema(
            schema_b_id.clone(),
            vec![import_resolved("schema_a", schema_a_id.clone())],
        );
        // C depends on A
        graph.add_schema(
            schema_c_id.clone(),
            vec![import_resolved("schema_a", schema_a_id.clone())],
        );
        // A has no dependencies
        graph.add_schema(schema_a_id.clone(), vec![]);

        graph.build_dependencies();

        // successors() returns "who depends on me"
        // A is depended on by both B and C
        let a_successors = graph.successors(&schema_a_id);
        assert_eq!(a_successors.len(), 2);
        assert!(a_successors.contains(&schema_b_id));
        assert!(a_successors.contains(&schema_c_id));

        // B is depended on by D
        let b_successors = graph.successors(&schema_b_id);
        assert_eq!(b_successors.len(), 1);
        assert!(b_successors.contains(&schema_d_id));

        // C is depended on by D
        let c_successors = graph.successors(&schema_c_id);
        assert_eq!(c_successors.len(), 1);
        assert!(c_successors.contains(&schema_d_id));

        // D has no dependents
        assert_eq!(graph.successors(&schema_d_id).len(), 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = SchemaDependencyGraph::new();

        let schema_a_id = schema_id("schema_a");
        let schema_b_id = schema_id("schema_b");

        // A depends on B
        graph.add_schema(
            schema_a_id.clone(),
            vec![import_resolved("schema_b", schema_b_id.clone())],
        );
        // B depends on A (circular!)
        graph.add_schema(
            schema_b_id.clone(),
            vec![import_resolved("schema_a", schema_a_id.clone())],
        );

        graph.build_dependencies();

        let cycle = vec![schema_a_id, schema_b_id];
        assert!(graph.has_circular_dependency(&cycle));
    }
}
