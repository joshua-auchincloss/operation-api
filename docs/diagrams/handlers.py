from diagrams import Diagram, Cluster, Edge
from diagrams.programming.language import Rust
from diagrams.generic.blank import Blank
from diagrams.generic.storage import Storage
from diagrams.programming.flowchart import InternalStorage
from docs.diagrams.common import graph_attr, node_attr, edge_attr

with Diagram(
    "Definition Type Hierarchy",
    filename="docs/diagrams/handlers",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    # Root AST
    ast_stream = Rust("AstStream\n(parsed *.pld)")

    with Cluster("Top-Level Items (Items)", graph_attr={"bgcolor": "#e8f4f8"}):
        namespace_decl = Storage("Namespace\nDeclaration")
        use_stmt = Storage("Use\nStatement")
        spanned_ns = Storage("Spanned\nNamespace\n(nested)")

        with Cluster("Type Definitions", graph_attr={"bgcolor": "#f0f8f8"}):
            struct_def = Rust("Struct\n(StructDef)")
            enum_def = Rust("Enum\n(EnumDef)")
            oneof_def = Rust("OneOf\n(OneOfDef)")
            error_def = Rust("Error\n(ErrorDef)")
            type_def = Rust("Type Alias\n(TypeDef)")
            operation_def = Rust("Operation\n(OperationDef)")

        ast_stream >> namespace_decl
        ast_stream >> use_stmt
        ast_stream >> spanned_ns
        ast_stream >> struct_def
        ast_stream >> enum_def
        ast_stream >> oneof_def
        ast_stream >> error_def
        ast_stream >> type_def
        ast_stream >> operation_def

    with Cluster("NamespaceChild Enum", graph_attr={"bgcolor": "#e8f8e8"}):
        ns_child_enum = Rust("NamespaceChild<'ctx>")

        child_namespace = Storage("Namespace(\nBox<NamespaceCtx>)")
        child_struct = Storage("Struct(\nStructDef)")
        child_enum = Storage("Enum(\nEnumDef)")
        child_oneof = Storage("OneOf(\nOneOfDef)")
        child_error = Storage("Error(\nErrorDef)")
        child_type = Storage("Type(\nTypeDef)")
        child_operation = Storage("Operation(\nOperationDef)")

        ns_child_enum >> child_namespace
        ns_child_enum >> child_struct
        ns_child_enum >> child_enum
        ns_child_enum >> child_oneof
        ns_child_enum >> child_error
        ns_child_enum >> child_type
        ns_child_enum >> child_operation

        struct_def >> Edge(label="wrapped") >> child_struct
        enum_def >> Edge(label="wrapped") >> child_enum
        oneof_def >> Edge(label="wrapped") >> child_oneof
        error_def >> Edge(label="wrapped") >> child_error
        type_def >> Edge(label="wrapped") >> child_type
        operation_def >> Edge(label="wrapped") >> child_operation
        spanned_ns >> Edge(label="wrapped") >> child_namespace

    with Cluster("Definition Enum (Type Resolution)", graph_attr={"bgcolor": "#f8e8e8"}):
        definition_enum = Rust("Definition<'ctx>")

        def_struct = Storage("Struct(\n&'ctx StructDef)")
        def_enum = Storage("Enum(\n&'ctx EnumDef)")
        def_oneof = Storage("OneOf(\n&'ctx OneOfDef)")
        def_error = Storage("Error(\n&'ctx ErrorDef)")
        def_alias = Storage("TypeAlias(\n&'ctx Type)")

        definition_enum >> def_struct
        definition_enum >> def_enum
        definition_enum >> def_oneof
        definition_enum >> def_error
        definition_enum >> def_alias

        child_struct >> Edge(label="borrowed ref", style="dashed") >> def_struct
        child_enum >> Edge(label="borrowed ref", style="dashed") >> def_enum
        child_oneof >> Edge(label="borrowed ref", style="dashed") >> def_oneof
        child_error >> Edge(label="borrowed ref", style="dashed") >> def_error
        child_type >> Edge(label="borrowed ref", style="dashed") >> def_alias

    with Cluster("Storage and Context", graph_attr={"bgcolor": "#e8f4f8"}):
        namespace_ctx = Rust("NamespaceCtx<'ctx>")

        with Cluster("Context Fields", graph_attr={"bgcolor": "#f0f8f8"}):
            ctx_ref = Storage("ctx: RefContext\n(package + namespace)")
            ctx_imports = Storage("imports:\nVec<UseDef>")
            ctx_children = Storage("children:\nBTreeMap<NamedItemContext,\nNamespaceChild>")
            ctx_registry = InternalStorage("registry:\nTypeRegistry<'ctx>")

        namespace_ctx >> ctx_ref
        namespace_ctx >> ctx_imports
        namespace_ctx >> ctx_children
        namespace_ctx >> ctx_registry

        child_namespace >> Edge(label="stored in") >> ctx_children
        use_stmt >> Edge(label="stored in") >> ctx_imports


# Type Resolution Handler Pipeline
with Diagram(
    "Type Resolution Handler Pipeline",
    filename="docs/diagrams/resolution_handlers",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    type_resolver = Rust("TypeResolver<'ctx>\n::new(namespace)")

    with Cluster("Phase 1: Anonymous Struct Extraction", graph_attr={"bgcolor": "#e8f4f8"}):
        traverse_children = Rust("from_child()\nTraverse Children")

        with Cluster("Handler Methods", graph_attr={"bgcolor": "#f0f8f8"}):
            handle_struct = Rust("from_struct_fields()\nStructDef → Fields")
            handle_type = Rust("from_type()\nTypeDef → Type")
            handle_operation = Rust("from_operation()\nOperationDef → Args/Return")
            handle_oneof = Rust("from_oneof()\nOneOfDef → Variants")

        with Cluster("Type Traversal", graph_attr={"bgcolor": "#f8f8f8"}):
            handle_anon_struct = Rust("anonymous_struct()\nType::Struct")
            handle_array = Rust("from_array()\nType::Array")
            handle_union = Rust("from_union()\nType::Union")
            handle_anon_oneof = Rust("from_anonymous_oneof()\nType::OneOf")

        type_resolver >> traverse_children
        traverse_children >> handle_struct
        traverse_children >> handle_type
        traverse_children >> handle_operation
        traverse_children >> handle_oneof

        handle_struct >> handle_anon_struct
        handle_type >> handle_anon_struct
        handle_operation >> handle_anon_struct
        handle_oneof >> handle_anon_struct

        handle_anon_struct >> Edge(label="recurse") >> handle_array
        handle_anon_struct >> Edge(label="recurse") >> handle_union
        handle_anon_struct >> Edge(label="recurse") >> handle_anon_oneof

    with Cluster("Union Processing", graph_attr={"bgcolor": "#e8f8e8"}):
        identify_unions = Rust("identify_unions()\nFind Union Types")
        validate_unions = Rust("validate_unions()\nCheck Operands")
        merge_unions = Rust("merge_unions()\nCreate Merged Structs")

        union_record = Storage("UnionRecord<'ns>\n- union_ref\n- context_stack\n- in_oneof")
        union_struct = Storage("Generated\nStructDef")

        identify_unions >> union_record >> validate_unions >> merge_unions >> union_struct

    with Cluster("Type Alias Resolution", graph_attr={"bgcolor": "#f8e8e8"}):
        resolve_aliases = Rust("resolve_type_aliases()\nFollow Alias Chains")

        with Cluster("Alias Handling", graph_attr={"bgcolor": "#f0f8f8"}):
            build_graph = Storage("Build Alias\nDependency Graph")
            detect_cycles = Rust("Detect Circular\nAliases")
            resolve_chain = Rust("Resolve to\nPrimitive Type")

        resolve_aliases >> build_graph >> detect_cycles >> resolve_chain

    with Cluster("Metadata Resolution", graph_attr={"bgcolor": "#e8f8e8"}):
        metadata_ctx = Rust("MetadataContext<'ns>\n::new(namespace)")

        with Cluster("Version Resolution", graph_attr={"bgcolor": "#f0f8f8"}):
            version_item = Storage("Item Meta\n(version)")
            version_ns = Storage("Namespace Inner\n(version)")
            version_default = Storage("Default: 1")
            resolve_version = Rust("resolve_version()\nPriority: item → ns → default")

            resolve_version >> Edge(label="priority 1") >> version_item
            resolve_version >> Edge(label="priority 2") >> version_ns
            resolve_version >> Edge(label="priority 3") >> version_default

        with Cluster("Error Type Resolution", graph_attr={"bgcolor": "#f0f8f8"}):
            error_op = Storage("Operation Meta\n(error)")
            error_ns = Storage("Namespace Inner\n(error)")
            error_default = Storage("Default: Error")
            resolve_error = Rust("resolve_error_type()\nPriority: op → ns → default")

            resolve_error >> Edge(label="priority 1") >> error_op
            resolve_error >> Edge(label="priority 2") >> error_ns
            resolve_error >> Edge(label="priority 3") >> error_default

        metadata_ctx >> resolve_version
        metadata_ctx >> resolve_error

    with Cluster("Reference Validation", graph_attr={"bgcolor": "#e8f8e8"}):
        validate_refs = Rust("validate_all_references()\nCheck All Type Refs")
        type_registry = InternalStorage("TypeRegistry\n::resolve()")

        with Cluster("Resolution Strategy", graph_attr={"bgcolor": "#f0f8f8"}):
            check_builtin = Storage("Check Builtins\n(i32, str, ...)")
            check_local = Storage("Check Local\nNamespace")
            check_imports = Storage("Check Imports\n(ImportRegistry)")
            check_parents = Storage("Walk Parent\nNamespaces")
            check_wildcard = Storage("Check Wildcard\nImports")

        validate_refs >> type_registry
        type_registry >> check_builtin
        type_registry >> Edge(label="fallback") >> check_local
        type_registry >> Edge(label="fallback") >> check_imports
        type_registry >> Edge(label="fallback") >> check_parents
        type_registry >> Edge(label="fallback") >> check_wildcard

    # Connect phases
    union_struct >> Edge(label="feeds") >> metadata_ctx
    resolve_chain >> Edge(label="feeds") >> metadata_ctx
    metadata_ctx >> Edge(label="feeds") >> validate_refs


# Context and Registry Relationships
with Diagram(
    "Context and Registry Relationships",
    filename="docs/diagrams/context_registry",
    show=False,
    direction="LR",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    with Cluster("CompileCtx", graph_attr={"bgcolor": "#e8f4f8"}):
        compile_ctx = Rust("CompileCtx<'ctx>")
        root_schema = Rust("root:\nSchemaCtx<'ctx>")
        dependencies = Storage("dependencies:\nBTreeMap<String,\nSchemaCtx>")
        global_registry = InternalStorage("type_registry:\nTypeRegistry<'ctx>")

        compile_ctx >> root_schema
        compile_ctx >> dependencies
        compile_ctx >> global_registry

    with Cluster("SchemaCtx", graph_attr={"bgcolor": "#e8f8e8"}):
        schema_ctx = Rust("SchemaCtx<'ctx>")
        package_manifest = Storage("package:\nPackageManifest")
        namespaces = Storage("namespaces:\nBTreeMap<String,\nNamespaceCtx>")
        schema_registry = InternalStorage("registry:\nTypeRegistry<'ctx>")

        schema_ctx >> package_manifest
        schema_ctx >> namespaces
        schema_ctx >> schema_registry

        root_schema >> Edge(label="is-a") >> schema_ctx
        dependencies >> Edge(label="contains") >> schema_ctx
        global_registry >> Edge(label="shared") >> schema_registry

    with Cluster("NamespaceCtx", graph_attr={"bgcolor": "#f8e8e8"}):
        ns_ctx = Rust("NamespaceCtx<'ctx>")
        ref_context = Storage("ctx:\nRefContext\n(package + namespace)")
        ns_imports = Storage("imports:\nVec<UseDef>")
        ns_children = Storage("children:\nBTreeMap<NamedItemContext,\nNamespaceChild>")
        ns_registry = InternalStorage("registry:\nTypeRegistry<'ctx>")

        ns_ctx >> ref_context
        ns_ctx >> ns_imports
        ns_ctx >> ns_children
        ns_ctx >> ns_registry

        namespaces >> Edge(label="contains") >> ns_ctx
        schema_registry >> Edge(label="shared") >> ns_registry

    with Cluster("TypeRegistry Internals", graph_attr={"bgcolor": "#e8f4f8"}):
        registry_struct = Rust("TypeRegistry<'ctx>")
        inner_map = Storage("inner:\nArc<Mutex<HashMap<\nNamedItemContext,\nResolvedType>>>")

        registry_struct >> inner_map

        ns_registry >> Edge(label="is-a") >> registry_struct

    with Cluster("Registration Flow", graph_attr={"bgcolor": "#e8f8e8"}):
        register_call = Rust("registry.register()\nAdd Type")
        fqn_key = Storage("NamedItemContext\n(FQN key)")
        resolved_type = Storage("ResolvedType<'ctx>\n- kind: Definition\n- qualified_path")

        register_call >> fqn_key >> inner_map
        register_call >> resolved_type >> inner_map

        ns_children >> Edge(label="iterate") >> register_call

    with Cluster("Resolution Flow", graph_attr={"bgcolor": "#e8f8e8"}):
        resolve_call = Rust("registry.resolve()\nFind Type")
        path_or_ident = Storage("PathOrIdent\n(reference)")
        import_registry = Storage("ImportRegistry\n(analyzed imports)")
        lookup_result = Storage("Option<\nResolvedType<'ctx>>")

        resolve_call >> path_or_ident
        resolve_call >> import_registry
        resolve_call >> inner_map >> lookup_result

        ns_imports >> Edge(label="build") >> import_registry


# FromNamedSource wrapper pattern
with Diagram(
    "FromNamedSource Pattern",
    filename="docs/diagrams/source_tracking",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    with Cluster("Source Tracking Pattern", graph_attr={"bgcolor": "#e8f4f8"}):
        trait_with_source = Rust("trait WithSource:\nBlanket impl for Sized")

        wrapper = Storage("FromNamedSource<T>\n- source: PathBuf\n- value: T")

        method = Rust("fn with_source(\n  self,\n  source: PathBuf\n) -> FromNamedSource<Self>")

        trait_with_source >> method >> wrapper

    with Cluster("Usage Examples", graph_attr={"bgcolor": "#e8f8e8"}):
        struct_usage = Storage("StructDef\n.with_source(path)")
        wrapped_struct = Storage("FromNamedSource<\nStructDef>")

        enum_usage = Storage("EnumDef\n.with_source(path)")
        wrapped_enum = Storage("FromNamedSource<\nEnumDef>")

        ns_usage = Storage("NamespaceChild\n.with_source(path)")
        wrapped_ns_child = Storage("FromNamedSource<\nNamespaceChild>")

        struct_usage >> wrapped_struct
        enum_usage >> wrapped_enum
        ns_usage >> wrapped_ns_child

        method >> Edge(label="example", style="dashed") >> struct_usage
        method >> Edge(label="example", style="dashed") >> enum_usage
        method >> Edge(label="example", style="dashed") >> ns_usage

    with Cluster("Error Reporting", graph_attr={"bgcolor": "#e8f8e8"}):
        error_context = Storage("Error with\n- span\n- source file\n- source content")

        extract_source = Rust("extract source:\nwrapper.source")
        extract_value = Rust("extract value:\nwrapper.value")
        build_miette = Rust("Build miette\nReport")

        wrapped_struct >> extract_source >> error_context
        wrapped_struct >> extract_value >> error_context
        error_context >> build_miette

    with Cluster("Type Aliases", graph_attr={"bgcolor": "#e8f8e8"}):
        named_comment = Storage("NamedCommentStream =\nFromNamedSource<\nCommentStream>")
        named_error_meta = Storage("NamedErrorMeta =\nFromNamedSource<\nErrorMeta>")
        named_version = Storage("NamedVersionMeta =\nFromNamedSource<\nVersionMeta>")
        named_ns_def = Storage("NamedNamespaceDef =\nFromNamedSource<\nNamespaceDef>")
        named_use = Storage("NamedUseDef =\nFromNamedSource<\nUseDef>")
        named_child = Storage("NamedNamespaceChild =\nFromNamedSource<\nNamespaceChild>")

        wrapper >> Edge(label="type alias", style="dashed") >> named_comment
        wrapper >> Edge(label="type alias", style="dashed") >> named_error_meta
        wrapper >> Edge(label="type alias", style="dashed") >> named_version
        wrapper >> Edge(label="type alias", style="dashed") >> named_ns_def
        wrapper >> Edge(label="type alias", style="dashed") >> named_use
        wrapper >> Edge(label="type alias", style="dashed") >> named_child


# New diagram: Parallel Type Registration Pipeline
with Diagram(
    "Parallel Type Registration Pipeline",
    filename="docs/diagrams/parallel_registration",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    entry = Rust("SchemaCompiler\n::compile_all()")

    with Cluster("Step 1: Schema Graph", graph_attr={"bgcolor": "#ffe8e8"}):
        build_graph = Rust("build_graph()\nCollect All Schemas")

        with Cluster("Graph Construction", graph_attr={"bgcolor": "#fff0f0"}):
            add_root = Storage("Add root schema")
            add_deps = Storage("Add dependencies\nfrom ctx")
            extract_imports = Rust("extract_imports()\nper schema")
            build_deps = Rust("build_dependencies()\nConnect edges")

        scc_check = Rust("strongly_connected_components()\nDetect cycles")
        topo_sort = Rust("topological_sort_into_groups()\nDependency order")

        entry >> build_graph
        build_graph >> add_root >> extract_imports
        build_graph >> add_deps >> extract_imports
        extract_imports >> build_deps >> scc_check >> topo_sort

    with Cluster("Step 2: Per-Schema Compilation", graph_attr={"bgcolor": "#e8ffe8"}):
        compile_schema = Rust("compile_schema()\nfor each schema")
        get_schema = Rust("Get SchemaCtx\nfrom ctx or deps")
        namespace_levels = Rust("namespace_levels()\nBFS discovery")

        with Cluster("Namespace Grouping", graph_attr={"bgcolor": "#f0fff0"}):
            depth0 = Storage("Depth 0\nroot namespaces")
            depth1 = Storage("Depth 1\nchildren")
            depthN = Storage("Depth N\ngrandchildren...")

            depth0 >> Edge(label="barrier", color="blue") >> depth1
            depth1 >> Edge(label="barrier", color="blue") >> depthN

        topo_sort >> compile_schema >> get_schema >> namespace_levels
        namespace_levels >> depth0

    with Cluster("Step 3: Per-Namespace Type Registration", graph_attr={"bgcolor": "#e8e8ff"}):
        register_recursive = Rust("register_types_recursive()\nfor each namespace")

        with Cluster("Type Dependency Analysis", graph_attr={"bgcolor": "#f0f0ff"}):
            extractor = Rust("TypeExtractor\n::extract_from_namespace()")

            with Cluster("Extraction Methods", graph_attr={"bgcolor": "#f8f8ff"}):
                from_struct = Rust("extract_from_struct()\nStruct fields")
                from_oneof = Rust("extract_from_oneof()\nOneOf variants")
                from_error = Rust("extract_from_error()\nError fields")
                from_type = Rust("extract_from_type()\nType alias")
                from_enum = Rust("extract_from_enum()\nEnum (no deps)")

            build_dep_graph = Storage("TypeDependencyGraph\n- nodes: TypeNode\n- edges: EdgeKind")

        with Cluster("Cycle Detection", graph_attr={"bgcolor": "#f0f0ff"}):
            scc_types = Rust("strongly_connected_components()\nFind cycles")
            check_edges = Rust("Check edge kinds\nin cycle")

            terminating = Storage("✅ Terminating\n(has Optional/Array)")
            non_terminating = Storage("❌ Non-Terminating\n(Required only)")

        with Cluster("Topological Registration", graph_attr={"bgcolor": "#f0f0ff"}):
            topo_types = Rust("topological_sort_into_groups()\nDependency groups")

            with Cluster("Registration Groups", graph_attr={"bgcolor": "#f8f8ff"}):
                group0 = Storage("Group 0\n[primitives, enums]")
                group1 = Storage("Group 1\n[depends on 0]")
                group2 = Storage("Group 2\n[depends on 0,1]")

                group0 >> Edge(label="barrier\n(try_join_all)", color="purple", style="bold") >> group1
                group1 >> Edge(label="barrier\n(try_join_all)", color="purple", style="bold") >> group2

            register_type = Rust("register_type()\nper type in group")
            actual_call = Rust("schema.registry\n.register()")

        depthN >> register_recursive >> extractor
        extractor >> from_struct >> build_dep_graph
        extractor >> from_oneof >> build_dep_graph
        extractor >> from_error >> build_dep_graph
        extractor >> from_type >> build_dep_graph
        extractor >> from_enum >> build_dep_graph

        build_dep_graph >> scc_types >> check_edges
        check_edges >> Edge(label="allow") >> terminating
        check_edges >> Edge(label="reject") >> non_terminating

        terminating >> topo_types >> group0
        topo_types >> register_type >> actual_call

    with Cluster("Step 4: Recursive Child Processing", graph_attr={"bgcolor": "#fff8e8"}):
        find_children = Rust("Find child namespaces\nat depth+1")
        parallel_children = Storage("Process children\nin parallel")

        actual_call >> find_children >> parallel_children
        parallel_children >> Edge(label="recursive", style="dashed", color="orange") >> register_recursive

    with Cluster("Key Data Structures", graph_attr={"bgcolor": "#f8f8e8"}):
        with Cluster("TypeDependency", graph_attr={"bgcolor": "#fcfcf0"}):
            dep_struct = Storage("TypeDependency:\n- target_candidates: Vec<NamedItemContext>\n- kind: EdgeKind\n- field_path: Vec<String>")

        with Cluster("EdgeKind", graph_attr={"bgcolor": "#fcfcf0"}):
            edge_required = Storage("Required\n(non-optional field)")
            edge_optional = Storage("Optional\n(optional field)")
            edge_array = Storage("Array\n(array element)")

        with Cluster("NamedItemContext", graph_attr={"bgcolor": "#fcfcf0"}):
            named_item = Storage("NamedItemContext:\n- context: RefContext\n- name: SpannedToken![ident]")
            fqn_example = Storage("Example:\npkg::namespace::TypeName")

        actual_call >> dep_struct
        dep_struct >> edge_required
        dep_struct >> edge_optional
        dep_struct >> edge_array
        actual_call >> named_item >> fqn_example
