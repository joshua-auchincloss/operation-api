# operation-api

```sh
go install github.com/go-task/task/v3/cmd/task@latest
```

your goal is to resolve the issues with capturing doc comments in the `macros` crate, as well as resolve missing `#[doc = ".."]` attributes in generated code in `core/generate/rust.rs`.

for example:

1. user derives Enum, Struct, OneOf, or Error
2. user has comments at either parent level or field / variant level
3. user generates code from resultant "Defined" / Definition.
4. generated code must have the same captured comments from the derived "Defined" implementation.

to fix the issues, you may want to trace usages of `Meta::doc_comment()` / `.meta.doc_comment()`, or types carrying the `description: Option<String>` field for generation debugging. to debug macro usage, trace uses of `Vec<Attribute>` -> `Option<DescOrPath>`
