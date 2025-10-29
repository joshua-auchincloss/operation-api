// Type dependency graph for topological registration
//
// This module implements the type dependency graph used for determining
// the correct order to register types. It handles cycles by distinguishing
// between terminating (optional/array) and non-terminating (required) edges.

use std::collections::{HashMap, HashSet};

use crate::ctx::paths::NamedItemContext;

/// Identifies an edge kind in the type dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Required field: field: T
    /// These edges form non-terminating cycles
    Required,

    /// Optional field: field?: T
    /// These edges can terminate recursion
    Optional,

    /// Array field: field: T[]
    /// These edges can terminate recursion (empty array)
    Array,
}

impl EdgeKind {
    /// Returns true if this edge can terminate a cycle
    pub fn is_terminating(self) -> bool {
        matches!(self, EdgeKind::Optional | EdgeKind::Array)
    }
}

/// A dependency from one type to another
///
/// Uses candidate-based resolution: stores multiple possible `NamedItemContext`
/// values that match the type reference, mirroring `TypeRegistry.resolve()` logic.
/// After topological registration, the exact reference is resolved by checking
/// which candidate exists in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDependency {
    pub target_candidates: Vec<NamedItemContext>,

    pub kind: EdgeKind,

    pub field_path: Vec<String>,
}

impl TypeDependency {
    /// Create a dependency with a single known target
    /// Used when the type is already resolved (e.g., local definitions)
    pub fn with_target(
        target: NamedItemContext,
        kind: EdgeKind,
        field_path: Vec<String>,
    ) -> Self {
        Self {
            target_candidates: vec![target],
            kind,
            field_path,
        }
    }

    /// Create a dependency with multiple candidate targets
    /// Used when resolving type references that may match imports
    pub fn with_candidates(
        candidates: Vec<NamedItemContext>,
        kind: EdgeKind,
        field_path: Vec<String>,
    ) -> Self {
        Self {
            target_candidates: candidates,
            kind,
            field_path,
        }
    }

    /// Resolve this dependency to an exact target using the registry
    /// Returns the first candidate that exists in the registry
    pub fn resolve_with_registry(
        &self,
        registry: &crate::ctx::registry::TypeRegistry,
    ) -> Option<NamedItemContext> {
        // Check each candidate in order until we find one that exists
        for candidate in &self.target_candidates {
            // If this candidate is registered, it's the resolved target
            if registry
                .all_types()
                .iter()
                .any(|(ctx, _)| ctx == candidate)
            {
                return Some(candidate.clone());
            }
        }
        None
    }
}

/// A node in the type dependency graph
#[derive(Debug, Clone)]
pub struct TypeNode {
    /// The fully qualified name of the type
    pub name: NamedItemContext,

    /// Dependencies on other types
    pub dependencies: Vec<TypeDependency>,
}

/// Type dependency graph for a single schema
///
/// This graph tracks dependencies between types within a schema.
/// It's used with pathfinding algorithms to:
/// 1. Detect cycles (strongly_connected_components)
/// 2. Determine registration order (topological_sort_into_groups)
///
/// **Important**: Types are identified by NamedItemContext (fully qualified paths)
/// rather than simple strings to support:
/// - Cross-namespace resolution (foo::User vs bar::User)
/// - Cross-schema dependencies
/// - Proper integration with TypeRegistry
#[derive(Debug, Clone, Default)]
pub struct TypeDependencyGraph {
    /// All type nodes indexed by fully qualified type name
    nodes: HashMap<NamedItemContext, TypeNode>,
}

impl TypeDependencyGraph {
    /// Create a new empty type dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add a type node to the graph
    pub fn add_type(
        &mut self,
        name: NamedItemContext,
        dependencies: Vec<TypeDependency>,
    ) {
        self.nodes
            .insert(name.clone(), TypeNode { name, dependencies });
    }

    /// Get all type names in the graph
    pub fn type_names(&self) -> Vec<NamedItemContext> {
        self.nodes.keys().cloned().collect()
    }

    /// Get successors for a type (for pathfinding algorithms)
    ///
    /// If `only_required` is true, only return Required edges.
    /// This is used for topological sorting where we want to ignore
    /// terminating cycles.
    ///
    /// Returns all candidate targets (not yet resolved to exact references).
    /// For graph algorithms, any candidate could be the actual dependency.
    ///
    /// **Important**: Only returns successors that actually exist in the graph.
    /// Cross-namespace/cross-package references are filtered out.
    pub fn successors(
        &self,
        type_name: &NamedItemContext,
        only_required: bool,
    ) -> Vec<NamedItemContext> {
        self.nodes
            .get(type_name)
            .map(|node| {
                node.dependencies
                    .iter()
                    .filter(|dep| {
                        if only_required {
                            dep.kind == EdgeKind::Required
                        } else {
                            true
                        }
                    })
                    .flat_map(|dep| dep.target_candidates.iter().cloned())
                    // CRITICAL: Only include successors that exist in this graph
                    // Cross-namespace/cross-package references will be validated later
                    .filter(|candidate| self.nodes.contains_key(candidate))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all successors (including terminating edges)
    pub fn all_successors(
        &self,
        type_name: &NamedItemContext,
    ) -> Vec<NamedItemContext> {
        self.successors(type_name, false)
    }

    /// Get only required successors (excluding terminating edges)
    pub fn required_successors(
        &self,
        type_name: &NamedItemContext,
    ) -> Vec<NamedItemContext> {
        self.successors(type_name, true)
    }

    /// Check if a cycle contains at least one terminating edge
    pub fn has_terminating_edge(
        &self,
        cycle: &[NamedItemContext],
    ) -> bool {
        let cycle_set: HashSet<_> = cycle.iter().cloned().collect();

        for type_name in cycle {
            if let Some(node) = self.nodes.get(type_name) {
                for dep in &node.dependencies {
                    // If any candidate from this dependency is part of the cycle and is terminating
                    if dep
                        .target_candidates
                        .iter()
                        .any(|candidate| cycle_set.contains(candidate))
                        && dep.kind.is_terminating()
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get the node for a type
    pub fn get_node(
        &self,
        type_name: &NamedItemContext,
    ) -> Option<&TypeNode> {
        self.nodes.get(type_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tst::test_ctx;

    #[test]
    fn test_simple_dependency() {
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Required,
                vec!["field_b".to_string()],
            )],
        );

        graph.add_type(test_ctx("B"), vec![]);

        assert_eq!(graph.all_successors(&test_ctx("A")), vec![test_ctx("B")]);
        assert_eq!(graph.all_successors(&test_ctx("B")).len(), 0);
    }

    #[test]
    fn test_terminating_cycle() {
        let mut graph = TypeDependencyGraph::new();

        // A has optional field of type B
        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Optional,
                vec!["b".to_string()],
            )],
        );

        // B has required field of type A
        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Required,
                vec!["a".to_string()],
            )],
        );

        let cycle = vec![test_ctx("A"), test_ctx("B")];
        assert!(graph.has_terminating_edge(&cycle));
    }

    #[test]
    fn test_non_terminating_cycle() {
        let mut graph = TypeDependencyGraph::new();

        // A has required field of type B
        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Required,
                vec!["b".to_string()],
            )],
        );

        // B has required field of type A
        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Required,
                vec!["a".to_string()],
            )],
        );

        let cycle = vec![test_ctx("A"), test_ctx("B")];
        assert!(!graph.has_terminating_edge(&cycle));
    }

    #[test]
    fn test_required_vs_all_successors() {
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![
                TypeDependency::with_target(
                    test_ctx("B"),
                    EdgeKind::Required,
                    vec!["b".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("C"),
                    EdgeKind::Optional,
                    vec!["c".to_string()],
                ),
                TypeDependency::with_target(test_ctx("D"), EdgeKind::Array, vec!["d".to_string()]),
            ],
        );

        let all = graph.all_successors(&test_ctx("A"));
        assert_eq!(all.len(), 3);

        let required = graph.required_successors(&test_ctx("A"));
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], test_ctx("B"));
    }
}
