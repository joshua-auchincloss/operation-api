# Enums

Enums declare a named set of variants. Variants may be bare or have a static value.

Syntax:

```pld
enum Color {
	Red, // 0
	Green, // 1
	Blue, // 2
}

// explicit values
enum CookiePreference {
    OptOut = 0,
    RequiredOnly = 1,
    All = 2,
}

// string values
enum Status {
    Requested = "R",
    Pending = "P",
    Completed = "C",

    Rejected = "X"
}
```

> [!TIP]
>
> - Variants are separated by commas, trailing comma allowed.
> - Values can be integers or strings; the enum’s kind is inferred from its variants.
> - Value types must be contigious across the same enum (i.e. all numbers or all strings).
