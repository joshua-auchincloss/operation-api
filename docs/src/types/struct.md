# Structs

Structs are named types with named fields and types.

Syntax is as follows:

```pld
struct Foo {
    // field `bar` with builtin type i32
    bar: i32,
    // field `quz` with unsized array of anonymous struct type { id: i64 }
    quz: { id: i64 }[],
    // field `baz` with unsized array of either str or i32 (per datum)
    baz?: (oneof str | i32)[],
}

struct BarWithNested {
    quz: {
        id: i64,
        description: str
    }[],
}
```
