# mers ![build status](https://github.com/Dummi26/mers/actions/workflows/rust.yml/badge.svg)

Mers is an experimental programming language inspired by high-level and scripting languages, but with error handling inspired by rust.

## Why mers?

mers has...

- the type-safety of statically typed languages
  + mers will not crash unless you `exit()` with a nonzero exit code or write flawed assumptions using the builtin `.assume*()` functions (Rust's `unwrap` or `expect`)
- the flexibility of dynamically typed languages
  + `x := if condition() { "my string" } else { 12 } // <- this is valid`
- "correctness" (this is subjective and I'll be happy to discuss some of these decisions with people)
  + there is no `null` / `nil`
  + all references are explicit: if you pass a list by value, the original list will *never* be modified in any way.
  + errors are normal values! (no special treatment)
- a flexible type system to easily represent these errors and any complex structure including recursive types:
  + nothing: `[]` (an empty tuple)
  + a string: `string`
  + two strings: `[string string]` (a tuple)
  + many strings: `[string ...]` (a list)
  + Either a string or nothing (Rust's `Option<String>`): `string/[]`
  + Either an int or an error: (Rust's `Result<isize, String>`): `int/string` (better: `int/Err(string)`)
- compile-time execution through (explicit) macro syntax: `!(mers {<code>})` or `!(mers "file")`

## How mers?

Mers is written in rust. If you have `cargo`, use the build script in `build_scripts/` to produce the executable.

Now, create a new text file (or choose one from the examples) and run it: `mers <file>`.

## Docs

[syntax cheat sheet](docs/syntax_cheat_sheet.md)

[intro](docs/intro.md)

[builtins](docs/builtins.md)

[statements](docs/statements.md)
