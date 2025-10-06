# Linting Rules
Rules are given a RuleGroup, default level, and fix level.
## group `comments`

### rule `newline_after_single_line_comment`
Terminate single-line comments with a newline token.
- Default level: info
- Fix: safe


## group `file`

### rule `eof_should_terminate_with_newline`
Files should be terminated with a single new line.
- Default level: info
- Fix: safe


## group `one_of`

### rule `parens_around_oneof_when_array`
Parenthesize `oneof` when applying array suffix, e.g., `(oneof a | b)[]`.
- Default level: warn
- Fix: safe


## group `paired_tokens`

### rule `parens_around_anonymous_struct`
Do not parenthesize anonymous structs. Prefer `{ ... }` over `({ ... })`.
- Default level: warn
- Fix: safe


## group `trailing_commas`

### rule `no_trailing_comma_before_rbrace`
Do not include a trailing comma in the final field before `}`.
- Default level: warn
- Fix: safe
