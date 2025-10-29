from diagrams import Diagram, Cluster, Edge
from diagrams.programming.language import Rust
from diagrams.generic.blank import Blank
from diagrams.generic.storage import Storage
from diagrams.onprem.vcs import Git
from diagrams.programming.flowchart import InternalStorage

from docs.diagrams.common import graph_attr, node_attr, edge_attr


with Diagram(
    "Operation-API Compilation Pipeline",
    filename="docs/diagrams/compilation",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    # Entry Point
    entry = Rust("Entry Point\n(from_entry_point)")

    with Cluster("Initialization", graph_attr={"bgcolor": "#e8f4f8"}):
        manifest = Storage("Load Package\nManifest\n(schema.toml)")
        registry_init = InternalStorage("Create Global\nTypeRegistry")
        cache_init = Storage("Initialize\nSchema Cache")

        entry >> Edge(label="1.1") >> manifest
        entry >> Edge(label="1.2") >> registry_init
        entry >> Edge(label="1.3") >> cache_init

    with Cluster("Root Schema Loading", graph_attr={"bgcolor": "#e8f8e8"}):
        lib_pld = Storage("Parse lib.pld\n(namespace + uses)")
        schema_ctx = Rust("SchemaCtx::from_path()")

        with Cluster("Parse Namespaces", graph_attr={"bgcolor": "#f0f8f0"}):
            tokenize = Rust("Tokenize\n(*.pld files)")
            parse_ast = Rust("Parse AST\n(AstStream)")
            ns_ctx = Rust("NamespaceCtx\nConstruction")

            tokenize >> parse_ast >> ns_ctx

        manifest >> lib_pld
        lib_pld >> schema_ctx
        schema_ctx >> tokenize

    with Cluster("Dependency Resolution", graph_attr={"bgcolor": "#f8f0e8"}):
        discover = Rust("Discover Direct\nDependencies\n(from use stmts)")
        task_queue = Storage("Compilation\nTask Queue\n(async channel)")

        with Cluster("Parallel Workers", graph_attr={"bgcolor": "#fff8f0"}):
            worker1 = Rust("Worker 1")
            worker2 = Rust("Worker 2")
            worker_n = Rust("Worker N\n(max_concurrent)")

        result_collector = Rust("Result Collector\n(async aggregator)")

        ns_ctx >> discover >> task_queue
        task_queue >> Edge(label="pull tasks") >> worker1
        task_queue >> Edge(label="pull tasks") >> worker2
        task_queue >> Edge(label="pull tasks") >> worker_n

        worker1 >> Edge(label="results") >> result_collector
        worker2 >> Edge(label="results") >> result_collector
        worker_n >> Edge(label="results") >> result_collector

    with Cluster("Dependency Processing", graph_attr={"bgcolor": "#f8e8f8"}):
        with Cluster("Per-Dependency Workflow", graph_attr={"bgcolor": "#fcf0fc"}):
            resolve_dep = Rust("Resolve Dependency\n(path/git/registry)")
            check_circular = Rust("Check Circular\nDependency")
            check_version = Rust("Version\nCompatibility")
            check_cache = Storage("Check Cache\n(by key)")

            load_or_cache = Rust("Load Fresh\nor Use Cached")

            with Cluster("Fresh Load", graph_attr={"bgcolor": "#fff0ff"}):
                load_manifest_dep = Storage("Load Dependency\nManifest")
                parse_schema = Rust("Parse Schema\n(SchemaCtx)")
                compute_hash = Rust("Compute Content\nHash (mutable)")

            lockfile_check = Storage("Validate Lockfile\nChecksum")

            discover_transitive = Rust("Discover Transitive\nDependencies")
            enqueue_transitive = Storage("Enqueue to\nTask Queue")

            resolve_dep >> check_circular >> check_version
            check_version >> check_cache >> load_or_cache
            load_or_cache >> load_manifest_dep >> parse_schema
            load_or_cache >> compute_hash
            parse_schema >> lockfile_check
            lockfile_check >> discover_transitive >> enqueue_transitive

        result_collector >> resolve_dep
        enqueue_transitive >> Edge(label="recursive", color="red", style="dashed") >> task_queue

    with Cluster("Type Resolution", graph_attr={"bgcolor": "#e8e8f8"}):
        type_resolver = Rust("TypeResolver\n::new(namespace)")

        with Cluster("8 Resolution Phases (Sequential)", graph_attr={"bgcolor": "#f0f0f8"}):
            phase1 = Rust("Phase 1:\nAnonymous Struct\nExtraction")
            phase2 = Rust("Phase 2:\nUnion\nIdentification")
            phase3 = Rust("Phase 3:\nType Alias\nResolution")
            phase4 = Rust("Phase 4:\nUnion\nValidation")
            phase5 = Rust("Phase 5:\nUnion\nMerging")
            phase6 = Rust("Phase 6:\nVersion\nResolution")
            phase7 = Rust("Phase 7:\nError Type\nResolution")
            phase8 = Rust("Phase 8:\nReference\nValidation")

            phase1 >> phase2 >> phase3 >> phase4 >> phase5
            phase5 >> Edge(label="parallel") >> phase6
            phase5 >> Edge(label="parallel") >> phase7
            phase6 >> phase8
            phase7 >> phase8

        result_collector >> Edge(label="after all deps loaded") >> type_resolver
        type_resolver >> phase1

    with Cluster("Parallel Type Registration (NEW)", graph_attr={"bgcolor": "#e8f8ff"}):
        schema_compiler = Rust("SchemaCompiler\n::compile_all()")

        with Cluster("Schema-Level Parallelism", graph_attr={"bgcolor": "#f0f5ff"}):
            build_schema_graph = Rust("Build Schema\nDependency Graph")
            schema_scc = Rust("Detect Circular\nSchema Dependencies")
            schema_topo = Rust("Topological Sort\ninto Groups")

            with Cluster("Schema Groups (Sequential)", graph_attr={"bgcolor": "#f5faff"}):
                schema_group0 = Storage("Group 0\n[bar-corp]")
                schema_group1 = Storage("Group 1\n[abc-corp]")

                schema_group0 >> Edge(label="waterfall barrier\n(try_join_all)", color="blue", style="bold") >> schema_group1

        with Cluster("Namespace-Level Parallelism", graph_attr={"bgcolor": "#f0f5ff"}):
            ns_bfs = Rust("BFS Namespace\nDiscovery")
            ns_levels = Storage("Depth-Based\nGrouping")

            with Cluster("Namespace Depths (Sequential)", graph_attr={"bgcolor": "#f5faff"}):
                ns_depth0 = Storage("Depth 0\n[root namespaces]")
                ns_depth1 = Storage("Depth 1\n[children]")
                ns_depth2 = Storage("Depth 2\n[grandchildren]")

                ns_depth0 >> Edge(label="waterfall barrier\n(try_join_all)", color="blue", style="bold") >> ns_depth1
                ns_depth1 >> Edge(label="waterfall barrier\n(try_join_all)", color="blue", style="bold") >> ns_depth2

        with Cluster("Type-Level Parallelism", graph_attr={"bgcolor": "#f0f5ff"}):
            extract_type_deps = Rust("Extract Type\nDependencies")
            type_scc = Rust("SCC Cycle\nDetection")
            check_terminating = Rust("Check Terminating\nCycles (optional/array)")
            type_topo = Rust("Topological Sort\ninto Groups")

            with Cluster("Type Groups (Sequential)", graph_attr={"bgcolor": "#f5faff"}):
                type_group0 = Storage("Group 0\n[no deps]")
                type_group1 = Storage("Group 1\n[depends on 0]")
                type_group2 = Storage("Group 2\n[depends on 0,1]")

                type_group0 >> Edge(label="waterfall barrier\n(try_join_all)", color="blue", style="bold") >> type_group1
                type_group1 >> Edge(label="waterfall barrier\n(try_join_all)", color="blue", style="bold") >> type_group2

            register_types = Rust("registry.register()\nActual Registration")

        phase8 >> schema_compiler
        schema_compiler >> build_schema_graph >> schema_scc >> schema_topo
        schema_topo >> Edge(label="for each schema") >> ns_bfs >> ns_levels
        ns_levels >> Edge(label="for each namespace") >> extract_type_deps
        extract_type_deps >> type_scc >> check_terminating >> type_topo
        type_topo >> register_types

    with Cluster("Import Resolution", graph_attr={"bgcolor": "#f8e8e8"}):
        analyze_imports = Rust("Analyze Imports\n(ImportRegistry)")

        with Cluster("Import Types", graph_attr={"bgcolor": "#fcf0f0"}):
            single_import = Storage("Single:\nuse Foo;")
            named_import = Storage("Named:\nuse pkg::Foo;")
            ns_import = Storage("Namespace:\nuse pkg::bar;")
            wildcard_import = Storage("Wildcard:\nuse pkg::*;")

        resolve_refs = Rust("Resolve Type\nReferences")
        validate_deps = Rust("Validate Cross-\nPackage Refs")

        phase8 >> analyze_imports
        analyze_imports >> single_import
        analyze_imports >> named_import
        analyze_imports >> ns_import
        analyze_imports >> wildcard_import

        single_import >> resolve_refs
        named_import >> resolve_refs
        ns_import >> resolve_refs
        wildcard_import >> resolve_refs

        resolve_refs >> validate_deps

    with Cluster("Lockfile Generation", graph_attr={"bgcolor": "#f0f8e8"}):
        prune_versions = Rust("Prune Duplicate\nVersions")
        collect_metadata = Storage("Collect Resolved\nMetadata")
        generate_lockfile = Git("Generate\nschema.lock")
        write_lockfile = Storage("Write to\nFilesystem")

        validate_deps >> prune_versions
        prune_versions >> collect_metadata >> generate_lockfile >> write_lockfile

    with Cluster("Final Output", graph_attr={"bgcolor": "#e8f8e8"}):
        compile_ctx = Rust("CompileCtx\n(ready for codegen)")
        type_registry_final = InternalStorage("Populated\nTypeRegistry")
        dependency_graph = Storage("Complete\nDependency Graph")

        write_lockfile >> compile_ctx
        validate_deps >> type_registry_final >> compile_ctx
        result_collector >> dependency_graph >> compile_ctx


# Create second diagram for Root vs Dependency differences
with Diagram(
    "Root Package vs Dependency Processing",
    filename="docs/diagrams/root_vs_dependency",
    show=False,
    direction="LR",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    with Cluster("Root Package", graph_attr={"bgcolor": "#e8f4f8"}):
        root_manifest = Storage("Read schema.toml\n(local)")
        root_lib = Storage("Parse lib.pld\n(namespace decl)")
        root_uses = Storage("Extract use\nstatements")
        root_schema = Storage("Load all\nnamespaces")
        root_resolve = Rust("Full Type\nResolution\n(all 8 phases)")
        root_lockfile_write = Git("Write/Update\nschema.lock")
        root_cache_write = Storage("Populate Cache\n(for deps)")

        root_manifest >> root_lib >> root_uses >> root_schema
        root_schema >> root_resolve >> root_lockfile_write
        root_resolve >> root_cache_write

    with Cluster("Dependency Package", graph_attr={"bgcolor": "#f8e8e8"}):
        dep_resolve = Rust("Resolve from\nManifest\n(path/git/registry)")
        dep_lockfile_read = Git("Read\nschema.lock\n(if exists)")
        dep_version_check = Rust("Version\nCompatibility\nCheck")
        dep_cache_check = Storage("Check Cache\n(by hash key)")
        dep_load_or_cache = Rust("Load Fresh\nor Cached")
        dep_schema = Storage("Load\nNamespaces\n(use-based)")
        dep_partial_resolve = Rust("Partial Type\nResolution\n(referenced only)")
        dep_transitive = Storage("Enqueue\nTransitive Deps")

        dep_resolve >> dep_lockfile_read >> dep_version_check
        dep_version_check >> dep_cache_check >> dep_load_or_cache
        dep_load_or_cache >> dep_schema >> dep_partial_resolve
        dep_partial_resolve >> dep_transitive

    # Show the key differences
    root_resolve >> Edge(label="All Types", color="green", style="bold") >> Blank()
    dep_partial_resolve >> Edge(label="Referenced Only", color="orange", style="bold") >> Blank()

    root_lockfile_write >> Edge(label="Authoritative", color="green", style="bold") >> Blank()
    dep_lockfile_read >> Edge(label="Merge/Validate", color="orange", style="bold") >> Blank()

    root_cache_write >> Edge(label="Populate", color="green", style="bold") >> Blank()
    dep_cache_check >> Edge(label="Consume", color="orange", style="bold") >> Blank()


# Create third diagram for cache and lockfile flow
with Diagram(
    "Caching and Lockfile Strategy",
    filename="docs/diagrams/cache_lockfile",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    with Cluster("Cache Key Computation", graph_attr={"bgcolor": "#e8f4f8"}):
        pkg_name = Storage("Package Name\n(snake_case)")
        version = Storage("Version\n(semver)")

        with Cluster("Mutability Check", graph_attr={"bgcolor": "#f0f8f8"}):
            immutable = Storage("Immutable\n(git/registry)")
            mutable = Storage("Mutable\n(path)")
            content_hash = Rust("Compute\nContent Hash\n(*.pld files)")

            mutable >> content_hash

        cache_key = Storage("CacheKey\n(pkg, ver, hash?)")

        pkg_name >> cache_key
        version >> cache_key
        immutable >> Edge(label="no hash") >> cache_key
        content_hash >> Edge(label="+ hash") >> cache_key

    with Cluster("Cache Lookup", graph_attr={"bgcolor": "#e8f8e8"}):
        lookup = Rust("Check Cache")
        hit = Storage("Cache Hit\n(return cached)")
        miss = Storage("Cache Miss\n(load fresh)")
        version_check_cache = Rust("Verify Version\nCompatibility")

        cache_key >> lookup
        lookup >> Edge(label="found") >> hit >> version_check_cache
        lookup >> Edge(label="not found") >> miss

        version_check_cache >> Edge(label="compatible", color="green") >> Blank()
        version_check_cache >> Edge(label="incompatible", color="red") >> miss

    with Cluster("Lockfile Validation", graph_attr={"bgcolor": "#f8e8f8"}):
        read_lock = Git("Read schema.lock\n(if exists)")
        extract_checksums = Storage("Extract Package\nChecksums")
        compare_checksum = Rust("Compare\nChecksums")

        with Cluster("Validation Result", graph_attr={"bgcolor": "#fcf0f0"}):
            match_result = Storage("Match:\nUse Locked Version")
            mismatch_result = Storage("Mismatch:\nMark Invalidated")
            missing_result = Storage("Missing:\nAdd to Lock")

        read_lock >> extract_checksums >> compare_checksum
        compare_checksum >> Edge(label="match", color="green") >> match_result
        compare_checksum >> Edge(label="mismatch", color="red") >> mismatch_result
        compare_checksum >> Edge(label="not in lock", color="orange") >> missing_result

    with Cluster("Lockfile Generation", graph_attr={"bgcolor": "#e8f4f8"}):
        collect = Storage("Collect All\nResolved Metadata")
        prune = Rust("Prune Compatible\nDuplicates")
        build_graph = Storage("Build Dependency\nGraph")
        serialize = Git("Serialize to\nschema.lock")

        collect >> prune >> build_graph >> serialize

        mismatch_result >> Edge(label="trigger rebuild", color="red") >> collect
        missing_result >> Edge(label="trigger rebuild", color="orange") >> collect


# New diagram: Three-Level Cascading Waterfall
with Diagram(
    "Three-Level Cascading Waterfall Parallelism",
    filename="docs/diagrams/cascading_waterfall",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
):
    entry = Rust("SchemaCompiler\n::compile_all()")

    with Cluster("Level 1: Schema Dependencies", graph_attr={"bgcolor": "#ffe8e8"}):
        with Cluster("Build Graph", graph_attr={"bgcolor": "#fff0f0"}):
            add_root = Storage("Add Root\nSchema")
            add_deps = Storage("Add All\nDependencies")
            build_edges = Rust("Build Import\nEdges")

            add_root >> build_edges
            add_deps >> build_edges

        scc_schemas = Rust("SCC: Detect\nCircular Schemas")
        topo_schemas = Rust("Topological Sort\ninto Groups")

        with Cluster("Compilation Groups (Sequential)", graph_attr={"bgcolor": "#fff8f8"}):
            with Cluster("Group 0: No Dependencies", graph_attr={"bgcolor": "#ffffff"}):
                schema_b1 = Rust("bar-corp\n(parallel)")
                schema_b2 = Rust("other-lib\n(parallel)")

                schema_b1 >> Edge(label="independent", style="invis") >> schema_b2

            barrier1 = Storage("⚡ Waterfall Barrier\ntry_join_all()")

            with Cluster("Group 1: Depends on Group 0", graph_attr={"bgcolor": "#ffffff"}):
                schema_a = Rust("abc-corp\n(uses bar-corp)")

        entry >> build_edges >> scc_schemas >> topo_schemas
        topo_schemas >> schema_b1
        topo_schemas >> schema_b2
        schema_b1 >> barrier1
        schema_b2 >> barrier1
        barrier1 >> Edge(label="all complete", color="blue", style="bold") >> schema_a

    with Cluster("Level 2: Namespace Hierarchy", graph_attr={"bgcolor": "#e8ffe8"}):
        compile_schema = Rust("compile_schema()\nfor each schema")
        bfs_discovery = Rust("BFS Namespace\nDiscovery")
        depth_groups = Storage("Group by\nDepth")

        with Cluster("Namespace Depths (Sequential)", graph_attr={"bgcolor": "#f8fff8"}):
            with Cluster("Depth 0: Root Namespaces", graph_attr={"bgcolor": "#ffffff"}):
                ns_root1 = Rust("pkg::foo\n(parallel)")
                ns_root2 = Rust("pkg::bar\n(parallel)")

                ns_root1 >> Edge(label="independent", style="invis") >> ns_root2

            barrier2 = Storage("⚡ Waterfall Barrier\ntry_join_all()")

            with Cluster("Depth 1: Children", graph_attr={"bgcolor": "#ffffff"}):
                ns_child1 = Rust("pkg::foo::baz\n(parallel)")
                ns_child2 = Rust("pkg::bar::qux\n(parallel)")

        schema_a >> compile_schema >> bfs_discovery >> depth_groups
        depth_groups >> ns_root1
        depth_groups >> ns_root2
        ns_root1 >> barrier2
        ns_root2 >> barrier2
        barrier2 >> Edge(label="parents complete", color="green", style="bold") >> ns_child1
        barrier2 >> Edge(label="parents complete", color="green", style="bold") >> ns_child2

    with Cluster("Level 3: Type Dependencies", graph_attr={"bgcolor": "#e8e8ff"}):
        register_recursive = Rust("register_types_recursive()\nfor each namespace")
        extract_deps = Rust("TypeExtractor\n::extract_from_namespace()")
        scc_types = Rust("SCC: Detect\nCycles")
        check_cycles = Rust("Check Terminating\nvs Non-Terminating")
        topo_types = Rust("Topological Sort\ninto Groups")

        with Cluster("Type Groups (Sequential)", graph_attr={"bgcolor": "#f8f8ff"}):
            with Cluster("Group 0: No Dependencies", graph_attr={"bgcolor": "#ffffff"}):
                type_enum = Rust("Status\nenum (parallel)")
                type_prim = Rust("UserId\ntype alias (parallel)")

            barrier3 = Storage("⚡ Waterfall Barrier\ntry_join_all()")

            with Cluster("Group 1: Depends on Group 0", graph_attr={"bgcolor": "#ffffff"}):
                type_user = Rust("User\nstruct { id: UserId }")

            barrier4 = Storage("⚡ Waterfall Barrier\ntry_join_all()")

            with Cluster("Group 2: Depends on Groups 0-1", graph_attr={"bgcolor": "#ffffff"}):
                type_admin = Rust("Admin\nstruct { user: User }")

        actual_registration = Rust("registry.register()\nActual Type Registration")

        ns_child1 >> register_recursive >> extract_deps
        extract_deps >> scc_types >> check_cycles >> topo_types
        topo_types >> type_enum
        topo_types >> type_prim
        type_enum >> barrier3
        type_prim >> barrier3
        barrier3 >> Edge(label="deps registered", color="purple", style="bold") >> type_user
        type_user >> barrier4
        barrier4 >> Edge(label="deps registered", color="purple", style="bold") >> type_admin
        type_admin >> actual_registration

    with Cluster("Key Properties", graph_attr={"bgcolor": "#f8f8e8"}):
        prop1 = Storage("✅ Parallel Within Groups\n(siblings independent)")
        prop2 = Storage("✅ Sequential Between Groups\n(waterfall barriers)")
        prop3 = Storage("✅ Dependencies Before Dependents\n(topological order)")
        prop4 = Storage("✅ Safe Arc<SchemaCtx> Access\n(no unsafe code)")

        actual_registration >> prop1
        actual_registration >> prop2
        actual_registration >> prop3
        actual_registration >> prop4
