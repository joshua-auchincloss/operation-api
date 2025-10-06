# Errors

Errors group named variants representing failure cases. Variants may carry anonymized structs or a path to a type.

Syntax:

```pld
enum IoErrorCode {
    PipeFail
}

struct IoError {
	code: IoErrorCode,
	message?: str,
}

error ServerError {
    Unknown {
        report_id: str
    },
    Io(IoError)
}
```

> [!TIP]
>
> - Variants are top-level declarations with the `error` keyword.
> - Fields use the same rules as struct fields, including optional `?`.
