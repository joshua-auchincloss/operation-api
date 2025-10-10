# Format

The formatter ensures consistent code style across operation protocol definition files.

## Configuration

Format configuration is stored in `op-fmt.toml`:

```toml
# maximum line width before wrapping (default: 120)
max_width = 120

# use tabs for indentation (default: true)
indent_with_tabs = true

# number of spaces/tabs per indent level (default: 4)
indent_width = 4

# preserve multiple adjacent blank lines (default: true)
preserve_adjacent_blank_lines = true
```

## Indentation

The formatter uses tabs by default for all indentation:

```pld
struct User {
	id: i64,
	name: str,
	nested: {
		field: i32,
	},
}
```

## Spacing

### Operators

Union types use `&` (space-ampersand-space) as separator:

```pld
type Combined = Foo & Bar & Baz;
```

Oneof types use `|` (space-pipe-space) as separator:

```pld
oneof Result {
	Ok(i32),
	Err(str),
}

type Value = oneof i32 | str | bool;
```

### Commas

Commas separate items in structs, enums, and variants. No trailing comma on the last item:

```pld
struct Point {
	x: i32,
	y: i32,
}

enum Status {
	Active = 1,
	Inactive = 2,
}
```

## Blank Lines

Single blank line between top-level items:

```pld
namespace foo;

struct A {
	field: i32,
}

struct B {
	field: str,
}

operation get() -> A;
```

## Comments

### Single-line

Single-line comments are preserved as-is:

```pld
// this is a comment
struct Foo {
	bar: i32,
}
```

### Multi-line

Multi-line comments with newlines are indented to match surrounding context:

```pld
/*
	Multi-line comment
	with indented content
*/
struct Foo {
	/*
		Field documentation
		spans multiple lines
	*/
	bar: i32,
}
```

Single-line multi-line comments remain on one line:

```pld
/* brief comment */ struct Foo {}
```

## Module-level Attributes

Module-level attributes appear before other items:

```pld
#![error(MyError)]
namespace foo;

error MyError {
	Unknown(never),
}
```

Item-level attributes apply to the following item:

```pld
#[version(1)]
struct Foo {
	bar: i32,
}
```
