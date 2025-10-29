// Graph data structures for parallel compilation
//
// This module contains the three core graph types used by the pathfinding-based
// parallel compiler:
//
// 1. SchemaDependencyGraph - For multi-schema compilation ordering
//    - SchemaId: CacheKey (package name + version + content hash)
//    - Aligns with the caching infrastructure
//
// 2. TypeDependencyGraph - For topological type registration within a schema
//    - Uses simple String type names (local to schema)
//    - Tracks EdgeKind (Required, Optional, Array) for cycle detection
//
// 3. GlobalReferenceGraph - For cross-schema reference validation
//    - TypeRef: NamedItemContext (fully-qualified type paths)
//    - Naturally handles local vs external via package field
//    - Consistent with TypeRegistry's internal representation

pub mod extract;
pub mod references;
pub mod schemas;
pub mod types;

pub use extract::TypeExtractor;
