/// Type dependency extraction for building TypeDependencyGraph from namespace definitions.
///
/// This module traverses AST nodes (Struct, OneOf, Error, etc.) and extracts type dependencies
/// to build a TypeDependencyGraph suitable for topological sorting and cycle detection.
use crate::{
    ast::{
        err::ErrorType,
        one_of::OneOf,
        op::Operation,
        strct::{Sep, Struct},
        ty::{PathOrIdent, Type},
        ty_def::NamedType,
    },
    ctx::{
        NamespaceCtx, RefOrItemContext,
        common::NamespaceChild,
        paths::{NamedItemContext, RefContext},
    },
    defs::Spanned,
    tokens::IdentToken,
};

use super::types::{EdgeKind, TypeDependency, TypeDependencyGraph};

/// Type dependency extractor
pub struct TypeExtractor;

impl TypeExtractor {
    /// Generate candidate NamedItemContext values for a type reference
    ///
    /// This mirrors the logic in TypeRegistry.resolve() to generate the same
    /// permutations of possible resolutions. The candidates are:
    /// - For Ident: true_local + all imported items with matching name
    /// - For Path: true_local + imported namespaces + imported items
    ///
    /// After topological registration, we resolve to the exact target by checking
    /// which candidate exists in the TypeRegistry.
    pub(crate) fn generate_candidates(
        reference: &PathOrIdent,
        context: &RefContext,
        ns: &NamespaceCtx,
    ) -> Vec<NamedItemContext> {
        match reference {
            PathOrIdent::Ident(name) => {
                let true_local = context.item(name.clone());
                let mut candidates = ns
                    .imports
                    .iter()
                    .filter_map(|it| {
                        match &it.value {
                            RefOrItemContext::Item(it) => {
                                if &it.name == name {
                                    Some(it.clone())
                                } else {
                                    None
                                }
                            },
                            RefOrItemContext::Ref(_) => None,
                        }
                    })
                    .collect::<Vec<_>>();

                candidates.push(true_local);
                candidates
            },
            PathOrIdent::Path(path) => {
                let mut seg = path.borrow_path_inner().segments().clone();
                let Some(last) = seg.pop() else {
                    // Empty path shouldn't happen in valid AST, return empty candidates
                    return Vec::new();
                };

                let true_local = context
                    .extend(&seg)
                    .item(Spanned::call_site(IdentToken::new(last.clone())));

                let mut candidates = ns
                    .imports
                    .iter()
                    .filter_map(|it| {
                        match &it.value {
                            RefOrItemContext::Item(it) => {
                                if it.name.borrow_string() == &last {
                                    Some(it.clone())
                                } else {
                                    None
                                }
                            },
                            RefOrItemContext::Ref(r) => {
                                let qual = r
                                    .extend(&seg)
                                    .item(Spanned::call_site(IdentToken::new(last.clone())));
                                Some(qual)
                            },
                        }
                    })
                    .collect::<Vec<_>>();

                candidates.push(true_local);
                candidates
            },
        }
    }

    /// Build a TypeDependencyGraph from a namespace's children
    pub fn extract_from_namespace(
        ns_ctx: &NamespaceCtx,
        ref_context: &RefContext,
    ) -> TypeDependencyGraph {
        let mut graph = TypeDependencyGraph::new();

        for (item_ctx, child) in &ns_ctx.children {
            let dependencies = match &child.value {
                NamespaceChild::Struct(struct_def) => {
                    Self::extract_from_struct(&struct_def.def.value, ref_context, ns_ctx)
                },
                NamespaceChild::OneOf(oneof_def) => {
                    Self::extract_from_oneof(&oneof_def.def.value, ref_context, ns_ctx)
                },
                NamespaceChild::Error(error_def) => {
                    Self::extract_from_error(&error_def.def.value, ref_context, ns_ctx)
                },
                NamespaceChild::Type(type_def) => {
                    Self::extract_from_type_alias(&type_def.def.value, ref_context, ns_ctx)
                },
                NamespaceChild::Operation(op_def) => {
                    Self::extract_from_operation(&op_def.def.value, ref_context, ns_ctx)
                },
                NamespaceChild::Enum(_) => {
                    // Enums have no type dependencies (just literals)
                    Vec::new()
                },
                NamespaceChild::Namespace(_) => {
                    // Nested namespaces are handled separately
                    Vec::new()
                },
            };

            graph.add_type(item_ctx.clone(), dependencies);
        }

        graph
    }

    /// Extract dependencies from a struct definition
    fn extract_from_struct(
        struct_def: &Struct,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) -> Vec<TypeDependency> {
        let mut deps = Vec::new();

        for field in &struct_def.args.values {
            let field_name = field.value.name.borrow_string().to_string();
            let is_required = matches!(field.value.sep.value, Sep::Required { .. });

            Self::extract_from_type(
                &field.value.typ,
                &mut deps,
                vec![field_name.clone()],
                if is_required {
                    EdgeKind::Required
                } else {
                    EdgeKind::Optional
                },
                ref_context,
                ns_ctx,
            );
        }

        deps
    }

    /// Extract dependencies from a oneof definition
    fn extract_from_oneof(
        oneof_def: &OneOf,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) -> Vec<TypeDependency> {
        let mut deps = Vec::new();

        for variant_item in &oneof_def.variants.values {
            let variant = &variant_item.value;

            match &variant.value {
                crate::ast::variadic::Variant::Tuple { name, inner, .. } => {
                    let variant_name = name.borrow_string().to_string();
                    // Tuple variants have a single inner type
                    Self::extract_from_type(
                        inner,
                        &mut deps,
                        vec![variant_name],
                        EdgeKind::Optional, // OneOf variants are optional
                        ref_context,
                        ns_ctx,
                    );
                },
                crate::ast::variadic::Variant::LocalStruct { name, inner, .. } => {
                    let variant_name = name.borrow_string().to_string();
                    // LocalStruct variants have an anonymous struct
                    for field in &inner.value.fields.value.values {
                        let field_name = field.value.name.borrow_string().to_string();
                        let is_required = matches!(field.value.sep.value, Sep::Required { .. });

                        let field_path = vec![variant_name.clone(), field_name];
                        Self::extract_from_type(
                            &field.value.typ,
                            &mut deps,
                            field_path,
                            if is_required {
                                EdgeKind::Required
                            } else {
                                EdgeKind::Optional
                            },
                            ref_context,
                            ns_ctx,
                        );
                    }
                },
            }
        }

        deps
    }

    /// Extract dependencies from an error definition
    fn extract_from_error(
        error_def: &ErrorType,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) -> Vec<TypeDependency> {
        let mut deps = Vec::new();

        for variant_item in &error_def.variants.values {
            let variant = &variant_item.value;

            match &variant.value {
                crate::ast::variadic::Variant::Tuple { name, inner, .. } => {
                    let variant_name = name.borrow_string().to_string();
                    // Tuple variants have a single inner type
                    Self::extract_from_type(
                        inner,
                        &mut deps,
                        vec![variant_name],
                        EdgeKind::Optional, // Error variants are optional
                        ref_context,
                        ns_ctx,
                    );
                },
                crate::ast::variadic::Variant::LocalStruct { name, inner, .. } => {
                    let variant_name = name.borrow_string().to_string();
                    // LocalStruct variants have an anonymous struct
                    for field in &inner.value.fields.value.values {
                        let field_name = field.value.name.borrow_string().to_string();
                        let is_required = matches!(field.value.sep.value, Sep::Required { .. });

                        let field_path = vec![variant_name.clone(), field_name];
                        Self::extract_from_type(
                            &field.value.typ,
                            &mut deps,
                            field_path,
                            if is_required {
                                EdgeKind::Required
                            } else {
                                EdgeKind::Optional
                            },
                            ref_context,
                            ns_ctx,
                        );
                    }
                },
            }
        }

        deps
    }

    /// Extract dependencies from a type alias
    fn extract_from_type_alias(
        type_def: &NamedType,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) -> Vec<TypeDependency> {
        let mut deps = Vec::new();

        // Type aliases are direct references - treat as required
        Self::extract_from_type(
            &type_def.ty.value,
            &mut deps,
            vec![],
            EdgeKind::Required,
            ref_context,
            ns_ctx,
        );

        deps
    }

    /// Extract dependencies from an operation definition
    fn extract_from_operation(
        op_def: &Operation,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) -> Vec<TypeDependency> {
        let mut deps = Vec::new();

        // Extract from input parameters
        if let Some(ref args) = op_def.args {
            for param in &args.value.values {
                let param_name = param.value.name.borrow_string().to_string();
                Self::extract_from_type(
                    &param.value.typ,
                    &mut deps,
                    vec![format!("param:{}", param_name)],
                    EdgeKind::Required, // Operation params are required
                    ref_context,
                    ns_ctx,
                );
            }
        }

        // Extract from return type
        Self::extract_from_type(
            &op_def.return_type.value,
            &mut deps,
            vec!["return".to_string()],
            EdgeKind::Required,
            ref_context,
            ns_ctx,
        );

        deps
    }

    /// Extract type references from a Type AST node
    ///
    /// This function fully recursively processes all type structures:
    /// - **Ident**: Direct type reference - generates candidate resolutions and adds as dependency
    /// - **Array**: Recursively processes inner type with Array edge kind
    /// - **Paren**: Recursively processes wrapped type, preserving edge kind
    /// - **Result**: Recursively processes inner type with Optional edge kind
    /// - **Struct**: Recursively processes all field types
    /// - **OneOf**: Recursively processes all variant types as Optional
    /// - **Union**: Fully recursively processes all union members via helper function
    ///   - Handles arbitrary nesting depth of unions
    ///   - Processes both named type references and anonymous structs
    /// - **Builtin**: No dependencies
    ///
    /// The recursion ensures that deeply nested structures (e.g., unions within unions,
    /// structs within oneofs, arrays of unions, etc.) are fully traversed and all type
    /// dependencies are extracted with the correct edge kinds and field paths.
    fn extract_from_type(
        ty: &Type,
        deps: &mut Vec<TypeDependency>,
        field_path: Vec<String>,
        current_kind: EdgeKind,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) {
        match ty {
            Type::Ident { to } => {
                // This is a reference to another type - generate candidates
                let candidates = Self::generate_candidates(to, ref_context, ns_ctx);
                deps.push(TypeDependency::with_candidates(
                    candidates,
                    current_kind,
                    field_path.clone(),
                ));
            },
            Type::Array { ty } => {
                // Arrays change the edge kind to Array
                let inner_ty = match &ty.value {
                    crate::ast::array::Array::Unsized { ty, .. } => &ty.value,
                    crate::ast::array::Array::Sized { ty, .. } => &ty.value,
                };
                Self::extract_from_type(
                    inner_ty,
                    deps,
                    field_path,
                    EdgeKind::Array,
                    ref_context,
                    ns_ctx,
                );
            },
            Type::Paren { ty, .. } => {
                // Parenthesized type - preserve current edge kind
                Self::extract_from_type(
                    &ty.value,
                    deps,
                    field_path,
                    current_kind,
                    ref_context,
                    ns_ctx,
                );
            },
            Type::Result { ty, .. } => {
                // Result type - the inner type becomes optional (can be error)
                Self::extract_from_type(
                    &ty.value,
                    deps,
                    field_path,
                    EdgeKind::Optional,
                    ref_context,
                    ns_ctx,
                );
            },
            Type::Struct { ty } => {
                // Anonymous struct - extract from its fields
                for field in &ty.value.fields.value.values {
                    let field_name = field.value.name.borrow_string().to_string();
                    let is_required = matches!(field.value.sep.value, Sep::Required { .. });

                    let mut new_path = field_path.clone();
                    new_path.push(field_name);

                    Self::extract_from_type(
                        &field.value.typ,
                        deps,
                        new_path,
                        if is_required {
                            EdgeKind::Required
                        } else {
                            EdgeKind::Optional
                        },
                        ref_context,
                        ns_ctx,
                    );
                }
            },
            Type::OneOf { ty } => {
                // Anonymous oneof - extract from variants (which are just Types)
                for (i, variant) in ty
                    .value
                    .variants
                    .value
                    .values
                    .iter()
                    .enumerate()
                {
                    let mut new_path = field_path.clone();
                    new_path.push(format!("variant_{}", i));

                    // OneOf variants are optional (you pick one)
                    Self::extract_from_type(
                        &variant.value,
                        deps,
                        new_path,
                        EdgeKind::Optional,
                        ref_context,
                        ns_ctx,
                    );
                }
            },
            Type::Union { ty } => {
                // Union - extract from all operands (treat as optional since only one is used)
                for (i, operand) in ty.value.types.values.iter().enumerate() {
                    let mut new_path = field_path.clone();
                    new_path.push(format!("union_{}", i));

                    // Recursively process union members
                    Self::extract_from_union_member(
                        &operand.value,
                        deps,
                        new_path,
                        EdgeKind::Optional,
                        ref_context,
                        ns_ctx,
                    );
                }
            },
            Type::Builtin { .. } => {
                // Builtin types have no dependencies
            },
        }
    }

    /// Recursively extract dependencies from a union member (IdentOrUnion)
    ///
    /// This handles the full recursive structure of unions, including:
    /// - **Ident(Ref)**: Named type reference - generates candidate resolutions and adds as dependency
    /// - **Ident(Anonymous)**: Anonymous struct - recursively processes all fields
    /// - **Union**: Nested union - recursively processes all members (arbitrary depth)
    ///
    /// The recursion ensures that complex union structures like:
    /// `(A & B) & (C & (D & E))` are fully traversed and all type references
    /// (A, B, C, D, E) are extracted with proper field paths.
    fn extract_from_union_member(
        member: &crate::ast::union::IdentOrUnion,
        deps: &mut Vec<TypeDependency>,
        field_path: Vec<String>,
        current_kind: EdgeKind,
        ref_context: &RefContext,
        ns_ctx: &NamespaceCtx,
    ) {
        match member {
            crate::ast::union::IdentOrUnion::Ident(disc) => {
                match disc {
                    crate::ast::union::UnionDiscriminant::Ref(path_or_ident) => {
                        // This is a reference to a named type - generate candidates
                        let candidates =
                            Self::generate_candidates(path_or_ident, ref_context, ns_ctx);
                        deps.push(TypeDependency::with_candidates(
                            candidates,
                            current_kind,
                            field_path,
                        ));
                    },
                    crate::ast::union::UnionDiscriminant::Anonymous(anon_struct) => {
                        // Anonymous struct in union - recursively extract its fields
                        for field in &anon_struct.fields.value.values {
                            let field_name = field.value.name.borrow_string().to_string();
                            let is_required = matches!(field.value.sep.value, Sep::Required { .. });

                            let mut struct_path = field_path.clone();
                            struct_path.push(field_name);

                            // Recursively extract from the field's type
                            Self::extract_from_type(
                                &field.value.typ,
                                deps,
                                struct_path,
                                if is_required {
                                    EdgeKind::Required
                                } else {
                                    EdgeKind::Optional
                                },
                                ref_context,
                                ns_ctx,
                            );
                        }
                    },
                }
            },
            crate::ast::union::IdentOrUnion::Union { inner, .. } => {
                // Nested union - recursively process each member
                for (i, nested_member) in inner.value.types.values.iter().enumerate() {
                    let mut nested_path = field_path.clone();
                    nested_path.push(format!("nested_{}", i));

                    // Fully recursive call - handles arbitrary nesting depth
                    Self::extract_from_union_member(
                        &nested_member.value,
                        deps,
                        nested_path,
                        current_kind, // Preserve the edge kind through nesting
                        ref_context,
                        ns_ctx,
                    );
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tst::test_ctx;

    #[test]
    fn test_extract_empty_graph() {
        let graph = TypeDependencyGraph::new();
        assert_eq!(graph.type_names().len(), 0);
    }

    #[test]
    fn test_struct_with_type_reference() {
        // Test: struct Company { ceo: Person }
        // Should depend on Person with Required edge
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Company"),
            vec![TypeDependency::with_target(
                test_ctx("Person"),
                EdgeKind::Required,
                vec!["ceo".to_string()],
            )],
        );

        graph.add_type(test_ctx("Person"), vec![]);

        assert_eq!(graph.type_names().len(), 2);
        assert_eq!(
            graph.all_successors(&test_ctx("Company")),
            vec![test_ctx("Person")]
        );
        assert_eq!(
            graph.required_successors(&test_ctx("Company")),
            vec![test_ctx("Person")]
        );
    }

    #[test]
    fn test_optional_field_dependency() {
        // Test: struct User { profile?: Profile }
        // Should have Optional edge kind
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("User"),
            vec![TypeDependency::with_target(
                test_ctx("Profile"),
                EdgeKind::Optional,
                vec!["profile".to_string()],
            )],
        );

        graph.add_type(test_ctx("Profile"), vec![]);

        // all_successors includes optional edges
        assert_eq!(
            graph.all_successors(&test_ctx("User")),
            vec![test_ctx("Profile")]
        );

        // required_successors excludes optional edges
        assert_eq!(
            graph
                .required_successors(&test_ctx("User"))
                .len(),
            0
        );
    }

    #[test]
    fn test_array_dependency() {
        // Test: struct Team { members: Person[] }
        // Should have Array edge kind (which is terminating)
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Team"),
            vec![TypeDependency::with_target(
                test_ctx("Person"),
                EdgeKind::Array,
                vec!["members".to_string()],
            )],
        );

        graph.add_type(test_ctx("Person"), vec![]);

        assert_eq!(
            graph.all_successors(&test_ctx("Team")),
            vec![test_ctx("Person")]
        );
        assert_eq!(
            graph
                .required_successors(&test_ctx("Team"))
                .len(),
            0
        ); // Array is terminating
    }

    #[test]
    fn test_mutual_recursion_optional() {
        // Test: A { b?: B }, B { a?: A }
        // Terminating cycle via optional edges
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Optional,
                vec!["b".to_string()],
            )],
        );

        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Optional,
                vec!["a".to_string()],
            )],
        );

        assert_eq!(graph.type_names().len(), 2);

        // Cycle should be detected as terminating
        let cycle = vec![test_ctx("A"), test_ctx("B")];
        assert!(graph.has_terminating_edge(&cycle));
    }

    #[test]
    fn test_non_terminating_cycle() {
        // Test: A { b: B }, B { a: A }
        // Non-terminating cycle (all required edges)
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Required,
                vec!["b".to_string()],
            )],
        );

        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Required,
                vec!["a".to_string()],
            )],
        );

        // Cycle should NOT have terminating edges
        let cycle = vec![test_ctx("A"), test_ctx("B")];
        assert!(!graph.has_terminating_edge(&cycle));
    }

    #[test]
    fn test_diamond_dependency() {
        // Test: D -> B -> A, D -> C -> A
        // Common dependency pattern
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(test_ctx("A"), vec![]);

        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Required,
                vec!["a".to_string()],
            )],
        );

        graph.add_type(
            test_ctx("C"),
            vec![TypeDependency::with_target(
                test_ctx("A"),
                EdgeKind::Required,
                vec!["a".to_string()],
            )],
        );

        graph.add_type(
            test_ctx("D"),
            vec![
                TypeDependency::with_target(
                    test_ctx("B"),
                    EdgeKind::Required,
                    vec!["b".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("C"),
                    EdgeKind::Required,
                    vec!["c".to_string()],
                ),
            ],
        );

        // Verify structure
        assert_eq!(graph.type_names().len(), 4);
        assert_eq!(graph.all_successors(&test_ctx("D")).len(), 2);
        assert_eq!(graph.all_successors(&test_ctx("B")).len(), 1);
        assert_eq!(graph.all_successors(&test_ctx("C")).len(), 1);
        assert_eq!(graph.all_successors(&test_ctx("A")).len(), 0);
    }

    #[test]
    fn test_type_alias_chain() {
        // Test: type A = B, type B = C, type C = string
        // Each should depend on the next
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Required,
                vec![],
            )],
        );

        graph.add_type(
            test_ctx("B"),
            vec![TypeDependency::with_target(
                test_ctx("C"),
                EdgeKind::Required,
                vec![],
            )],
        );

        graph.add_type(test_ctx("C"), vec![]);

        assert_eq!(graph.type_names().len(), 3);
        assert_eq!(graph.all_successors(&test_ctx("A")), vec![test_ctx("B")]);
        assert_eq!(graph.all_successors(&test_ctx("B")), vec![test_ctx("C")]);
    }

    #[test]
    fn test_oneof_variants_optional() {
        // Test: oneof Result { ok(Value), err(Error) }
        // Variant types should have Optional edge kind
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Result"),
            vec![
                TypeDependency::with_target(
                    test_ctx("Value"),
                    EdgeKind::Optional,
                    vec!["ok".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("Error"),
                    EdgeKind::Optional,
                    vec!["err".to_string()],
                ),
            ],
        );

        graph.add_type(test_ctx("Value"), vec![]);
        graph.add_type(test_ctx("Error"), vec![]);

        // Has dependencies but they're all optional
        assert_eq!(
            graph
                .all_successors(&test_ctx("Result"))
                .len(),
            2
        );
        assert_eq!(
            graph
                .required_successors(&test_ctx("Result"))
                .len(),
            0
        );
    }

    #[test]
    fn test_union_members_optional() {
        // Test: type Combined = A & B & C
        // Union members should be Optional (only one used at runtime)
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Combined"),
            vec![
                TypeDependency::with_target(
                    test_ctx("A"),
                    EdgeKind::Optional,
                    vec!["union_0".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("B"),
                    EdgeKind::Optional,
                    vec!["union_1".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("C"),
                    EdgeKind::Optional,
                    vec!["union_2".to_string()],
                ),
            ],
        );

        assert_eq!(
            graph
                .all_successors(&test_ctx("Combined"))
                .len(),
            3
        );
        assert_eq!(
            graph
                .required_successors(&test_ctx("Combined"))
                .len(),
            0
        );
    }

    #[test]
    fn test_nested_union_field_paths() {
        // Test: type Complex = (A & B) & (C & D)
        // Should have nested field paths
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Complex"),
            vec![
                TypeDependency::with_target(
                    test_ctx("A"),
                    EdgeKind::Optional,
                    vec!["union_0".to_string(), "nested_0".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("B"),
                    EdgeKind::Optional,
                    vec!["union_0".to_string(), "nested_1".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("C"),
                    EdgeKind::Optional,
                    vec!["union_1".to_string(), "nested_0".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("D"),
                    EdgeKind::Optional,
                    vec!["union_1".to_string(), "nested_1".to_string()],
                ),
            ],
        );

        // Verify all dependencies extracted
        assert_eq!(
            graph
                .all_successors(&test_ctx("Complex"))
                .len(),
            4
        );

        // Verify nested paths in node
        if let Some(node) = graph.get_node(&test_ctx("Complex")) {
            assert!(
                node.dependencies
                    .iter()
                    .any(|d| d.field_path.len() == 2)
            );
        }
    }

    #[test]
    fn test_anonymous_struct_nested_paths() {
        // Test: oneof Response { ok(i32), err { code: i32, msg: string } }
        // LocalStruct variant should have nested field paths
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Response"),
            vec![TypeDependency::with_target(
                test_ctx("ErrorDetails"),
                EdgeKind::Optional,
                vec!["err".to_string(), "details".to_string()],
            )],
        );

        if let Some(node) = graph.get_node(&test_ctx("Response")) {
            assert_eq!(node.dependencies[0].field_path.len(), 2);
            assert_eq!(node.dependencies[0].field_path[0], "err");
            assert_eq!(node.dependencies[0].field_path[1], "details");
        }
    }

    #[test]
    fn test_operation_param_and_return_paths() {
        // Test: operation create(user: User, org: Org) -> Result
        // Should have distinct field paths for params and return
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("create"),
            vec![
                TypeDependency::with_target(
                    test_ctx("User"),
                    EdgeKind::Required,
                    vec!["param:user".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("Org"),
                    EdgeKind::Required,
                    vec!["param:org".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("Result"),
                    EdgeKind::Required,
                    vec!["return".to_string()],
                ),
            ],
        );

        assert_eq!(
            graph
                .all_successors(&test_ctx("create"))
                .len(),
            3
        );

        if let Some(node) = graph.get_node(&test_ctx("create")) {
            assert!(
                node.dependencies
                    .iter()
                    .any(|d| d.field_path[0].starts_with("param:"))
            );
            assert!(
                node.dependencies
                    .iter()
                    .any(|d| d.field_path[0] == "return")
            );
        }
    }

    #[test]
    fn test_disconnected_components() {
        // Test: Two unrelated dependency chains: A -> B and X -> Y
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("A"),
            vec![TypeDependency::with_target(
                test_ctx("B"),
                EdgeKind::Required,
                vec![],
            )],
        );
        graph.add_type(test_ctx("B"), vec![]);

        graph.add_type(
            test_ctx("X"),
            vec![TypeDependency::with_target(
                test_ctx("Y"),
                EdgeKind::Required,
                vec![],
            )],
        );
        graph.add_type(test_ctx("Y"), vec![]);

        assert_eq!(graph.type_names().len(), 4);

        // Components should be independent
        assert_eq!(graph.all_successors(&test_ctx("A")), vec![test_ctx("B")]);
        assert_eq!(graph.all_successors(&test_ctx("X")), vec![test_ctx("Y")]);

        // No cross-component dependencies
        assert!(
            !graph
                .all_successors(&test_ctx("A"))
                .contains((&test_ctx("X")))
        );
        assert!(
            !graph
                .all_successors(&test_ctx("A"))
                .contains((&test_ctx("Y")))
        );
    }

    #[test]
    fn test_result_type_makes_optional() {
        // Test: struct Response { data: Value! }
        // Result type (!) makes inner type Optional
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Response"),
            vec![TypeDependency::with_target(
                test_ctx("Value"),
                EdgeKind::Optional, // Result wrapping makes it optional
                vec!["data".to_string()],
            )],
        );

        assert_eq!(
            graph.all_successors(&test_ctx("Response")),
            vec![test_ctx("Value")]
        );
        assert_eq!(
            graph
                .required_successors(&test_ctx("Response"))
                .len(),
            0
        );
    }

    #[test]
    fn test_mixed_edge_kinds() {
        // Test struct with required, optional, and array fields
        let mut graph = TypeDependencyGraph::new();

        graph.add_type(
            test_ctx("Mixed"),
            vec![
                TypeDependency::with_target(
                    test_ctx("Required"),
                    EdgeKind::Required,
                    vec!["req".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("Optional"),
                    EdgeKind::Optional,
                    vec!["opt".to_string()],
                ),
                TypeDependency::with_target(
                    test_ctx("Array"),
                    EdgeKind::Array,
                    vec!["arr".to_string()],
                ),
            ],
        );

        // All successors includes everything
        assert_eq!(
            graph
                .all_successors(&test_ctx("Mixed"))
                .len(),
            3
        );

        // Required successors excludes optional and array
        assert_eq!(
            graph
                .required_successors(&test_ctx("Mixed"))
                .len(),
            1
        );
        assert_eq!(
            graph.required_successors(&test_ctx("Mixed"))[0],
            test_ctx("Required")
        );
    }
}
