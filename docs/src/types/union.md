# union

Unions compose multiple named types with `&`, creating an intersection/product of fields.

Syntax:

```pld
User & Permissions

// with grouping
User & (Admin & Audit)
```

## Resolution Priority

Priority is given left-to-right in case of conflicting fields.

e.g.

```pld
struct Foo {
    a: i32,
    b: str
}

struct Baz {
    a: i64
}


type Qux = Foo & Bar; /*
{
    a: i32,
    b: str
}
*/
```

> [!TIP]
>
> - Each side is an identifier or a parenthesized union.
> - `&` binds left-to-right; use parentheses to control grouping.
