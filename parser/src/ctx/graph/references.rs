use std::collections::HashMap;

use crate::ctx::{cache::CacheKey, paths::NamedItemContext};

#[derive(Debug, Clone)]
pub struct ReferenceNode {
    pub type_ref: NamedItemContext,

    pub schema_id: CacheKey,
    pub referenced_types: Vec<NamedItemContext>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalReferenceGraph {
    nodes: HashMap<NamedItemContext, ReferenceNode>,
}

impl GlobalReferenceGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_type(
        &mut self,
        type_ref: NamedItemContext,
        schema_id: CacheKey,
        referenced_types: Vec<NamedItemContext>,
    ) {
        self.nodes.insert(
            type_ref.clone(),
            ReferenceNode {
                type_ref,
                schema_id,
                referenced_types,
            },
        );
    }

    pub fn type_refs(&self) -> Vec<NamedItemContext> {
        self.nodes.keys().cloned().collect()
    }

    /// Returns weighted edges where the weight is always 1 (unweighted graph).
    pub fn successors(
        &self,
        type_ref: &NamedItemContext,
    ) -> Vec<(NamedItemContext, usize)> {
        self.nodes
            .get(type_ref)
            .map(|node| {
                node.referenced_types
                    .iter()
                    .map(|ref_type| (ref_type.clone(), 1))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_node(
        &self,
        type_ref: &NamedItemContext,
    ) -> Option<&ReferenceNode> {
        self.nodes.get(type_ref)
    }

    pub fn contains(
        &self,
        type_ref: &NamedItemContext,
    ) -> bool {
        self.nodes.contains_key(type_ref)
    }

    pub fn types_in_schema(
        &self,
        schema_id: &CacheKey,
    ) -> Vec<&ReferenceNode> {
        self.nodes
            .values()
            .filter(|node| &node.schema_id == schema_id)
            .collect()
    }

    pub fn leaf_types(&self) -> Vec<NamedItemContext> {
        self.nodes
            .values()
            .filter(|node| node.referenced_types.is_empty())
            .map(|node| node.type_ref.clone())
            .collect()
    }

    pub fn root_types(&self) -> Vec<NamedItemContext> {
        let referenced: std::collections::HashSet<_> = self
            .nodes
            .values()
            .flat_map(|node| node.referenced_types.iter())
            .collect();

        self.nodes
            .keys()
            .filter(|type_ref| !referenced.contains(type_ref))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ctx::{cache::CacheKey, paths::RefContext},
        defs::Spanned,
        tokens::IdentToken,
    };
    use operation_api_manifests::version::Version;

    fn schema_id(name: &str) -> CacheKey {
        CacheKey::new(name.to_string(), Version::parse("1.0.0").unwrap(), None)
    }

    fn type_ref(
        package: &str,
        namespace: &[&str],
        name: &str,
    ) -> NamedItemContext {
        let ns = namespace
            .iter()
            .map(|s| s.to_string())
            .collect();
        let context = RefContext::new(package.to_string(), ns);
        NamedItemContext::new(
            Spanned::call_site(IdentToken::new(name.to_string())),
            context,
        )
    }

    fn local_type_ref(name: &str) -> NamedItemContext {
        type_ref("schema_a", &[], name)
    }

    fn external_type_ref(
        package: &str,
        name: &str,
    ) -> NamedItemContext {
        type_ref(package, &[], name)
    }

    #[test]
    fn test_local_reference() {
        let mut graph = GlobalReferenceGraph::new();

        let user_ref = local_type_ref("User");

        graph.add_type(user_ref.clone(), schema_id("schema_a"), vec![]);

        assert!(graph.contains(&user_ref));
        assert_eq!(graph.type_refs().len(), 1);
    }

    #[test]
    fn test_cross_schema_reference() {
        let mut graph = GlobalReferenceGraph::new();

        let user_ref = local_type_ref("User");
        let address_ref = external_type_ref("schema_b", "Address");

        graph.add_type(
            user_ref.clone(),
            schema_id("schema_a"),
            vec![address_ref.clone()],
        );

        graph.add_type(address_ref.clone(), schema_id("schema_b"), vec![]);

        let successors = graph.successors(&user_ref);
        assert_eq!(successors.len(), 1);
        assert_eq!(successors[0].0, address_ref);
        assert_eq!(successors[0].1, 1); // Weight
    }

    #[test]
    fn test_leaf_and_root_types() {
        let mut graph = GlobalReferenceGraph::new();

        let a = local_type_ref("A");
        let b = local_type_ref("B");
        let c = local_type_ref("C");

        // A -> B -> C
        graph.add_type(a.clone(), schema_id("schema"), vec![b.clone()]);
        graph.add_type(b.clone(), schema_id("schema"), vec![c.clone()]);
        graph.add_type(c.clone(), schema_id("schema"), vec![]);

        let leaves = graph.leaf_types();
        assert_eq!(leaves.len(), 1);
        assert!(leaves.contains(&c));

        let roots = graph.root_types();
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&a));
    }

    #[test]
    fn test_types_in_schema() {
        let mut graph = GlobalReferenceGraph::new();

        let schema_a = schema_id("schema_a");
        let schema_b = schema_id("schema_b");

        graph.add_type(local_type_ref("User"), schema_a.clone(), vec![]);
        graph.add_type(local_type_ref("Post"), schema_a.clone(), vec![]);
        graph.add_type(
            external_type_ref("schema_b", "Address"),
            schema_b.clone(),
            vec![],
        );

        let types_a = graph.types_in_schema(&schema_a);
        assert_eq!(types_a.len(), 2);

        let types_b = graph.types_in_schema(&schema_b);
        assert_eq!(types_b.len(), 1);
    }
}
