# oneof

Represents a value that may be one of several types, separated by `|`.

Syntax:

```pld
// anonymous oneof
oneof i32 | str | bool

// nested with arrays and parens
(oneof i32 | f32)[]
```

> [!TIP]
>
> - Variants are parsed in order and preserved; trailing pipes aren’t allowed.
> - Parentheses group complex variants when combining with arrays or other constructs.
