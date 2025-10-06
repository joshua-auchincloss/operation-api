# operation

Represents a remote operation, with input args and a return type. Operations may return a result type, where the returned response may be an `error` defined within an operation or namespace.

Syntax:

```pld
namespace foo;

error MyError {
    Unknown { desc: str },
}

// operation add accepts `a` (`i32`) and `b` (`i32`), and returns i32 (infallible)
operation add(a: i32, b: i32) -> i32;

// operation try_sub accepts `value` (`i32`) and `sub` (`i32`), and returns a result i32 (`i32!`)
#[error(MyError)]
operation try_sub(value: i32, sub: i32) -> i32!;
```

> [!TIP]
>
> - You must provide the error type either as a namespace meta attribute (`#![error(MyError)]`) or an item attribute (`#[error(MyError)]`)
