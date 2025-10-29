# Schemas

Schemas are packaged, distributable collections of type definitions that can be shared across projects and teams. The schema system provides dependency management, versioning, and tree-shaking to optimize compilation.

> [!NOTE]
> Every schema package must have a `schema.toml` manifest and a `schema/lib.pld` entry point.

## Package Distribution

Schemas can be published and consumed as packages, similar to modules in other language ecosystems. Each package:

- Contains type definitions (`struct`, `enum`, `operation`, etc.)
- Declares dependencies on other packages
- Specifies a semantic version for compatibility tracking
- Can be distributed via Git, local paths, or registries

## Tree Shaking

The compiler performs automatic tree shaking to include only the definitions actually used by your project:

```plaintext
# In your schema
use bar_corp::baz;

struct MyMessage {
    value: baz::BazOrString,  # Only BazOrString is included
}
```

Only `BazOrString` and its transitive dependencies are compiled. Unused types from `bar_corp` are excluded, reducing compilation time and generated code size.

> [!TIP]
> Tree shaking happens automatically—you don't need to configure anything. The compiler tracks which types are actually referenced and only compiles those.

## Dependency Resolution

Dependencies are resolved at compile time:

1. Parse root package manifest
2. Load direct dependencies
3. Recursively resolve transitive dependencies
4. Detect circular dependencies
5. Build compilation graph with only referenced items

## Lockfile

A `schema.lock` file (planned) will ensure reproducible builds by pinning:

- Exact versions of all transitive dependencies
- Content checksums for integrity verification
- The specific items imported from each package

This enables:

- Consistent builds across environments
- Offline compilation with cached dependencies
- Detection of breaking changes in dependencies

> [!IMPORTANT]
> When implemented, always commit `schema.lock` to version control to ensure all team members and CI/CD pipelines use identical dependency versions.

## Example Ecosystem

```
abc-corp/              # Your package
├── schema.toml        # Manifest
└── schema/
    └── lib.pld        # Entry point

bar-corp/              # Dependency
├── schema.toml
└── schema/
    └── lib.pld

your-project/
├── schema.toml        # Depends on abc-corp
└── schema/
    ├── lib.pld
    └── types/
        └── messages.pld
```

Each package is independently versioned and can be developed, tested, and distributed separately.
