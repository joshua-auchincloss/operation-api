# Spanned Tokens

> [!TIP]
> Paired token spans refers to a block of content which has a clear start and end boundary. E.g. `{..}` (braced), `(..)` (parenthesized), `[..]` (bracketed).

## Paired Token Spans

Nested paired token spans should be split between new lines for each item, with increasing tab indentation to display nesting levels. Root token spans (e.g. top-level declarations) are separated by new lines and indented. Operators are trailing.

e.g.

```pld
// top level declaration
type Abc = (
    i32 |
    i64
);

message Foo {
    // single tab for brace
    field_a: i32,
    field_b: (
        // double tab for braced -> parenthesized
        abc |
        u32 |
        u64 |
        str
    )
};

error ServerError {
    Internal(str)
}
```
