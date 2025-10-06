# Builtin
| Token      | Description                                                                                                                                  |
|:-----------|:---------------------------------------------------------------------------------------------------------------------------------------------|
| `bool`     | a boolean type (true | false)                                                                                                                |
| `str`      | a string type                                                                                                                                |
| `i8`       | signed 8-bit integer                                                                                                                         |
| `i16`      | signed 16-bit integer                                                                                                                        |
| `i32`      | signed 32-bit integer                                                                                                                        |
| `i64`      | signed 64-bit integer                                                                                                                        |
| `u8`       | unsigned 8-bit integer                                                                                                                       |
| `u16`      | unsigned 16-bit integer                                                                                                                      |
| `u32`      | unsigned 32-bit integer                                                                                                                      |
| `u64`      | unsigned 64-bit integer                                                                                                                      |
| `f16`      | a signed 16-bit floating point number                                                                                                        |
| `f32`      | a signed 32-bit floating point number                                                                                                        |
| `f64`      | a signed 64-bit floating point number                                                                                                        |
| `complex`  | a complex number with real and imaginary parts.                                                                                              |
| `datetime` | a [iso 8601](https://en.wikipedia.org/wiki/ISO_8601) compliant datetime providing timezone.                                                  |
| `never`    | a unit type (0 size)                                                                                                                         |
| `binary`   | a binary stream. this is distinct from u8[], where we may have language specific types to utilize if you intend to manipulate octal streams. |