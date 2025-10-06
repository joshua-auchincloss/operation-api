# Type Alias

Defines a named type alias using `type Name = <type>;`.

Syntax:

```pld
type Id = i64;
type Name = str;
type MaybeId = i64!;        // result type (may raise)
type Pair = oneof i32 | str;
type Both = User & Permissions; // union
type List = (oneof i32 | f32)[]; // parenthesis is required to distinguish we have an array of oneof instead of a `f32[]`
```

## forms:

- Builtins: i8/i16/i32/i64, u8/u16/u32/u64, usize, f16/f32/f64, bool, str, datetime, complex, binary, never.
- Ident: references a named type (`User`).
- Oneof: `oneof A | B | C`.
- Union: `A & B` (identifiers or parenthesized unions).
- Arrays: `T[]` or `T[n]` (suffix form, can repeat).
- Parens: `(T)` to control precedence.
- Result: `T!` to indicate a type that may raise.
