# mers

Mers is a high-level programming language.
It is designed to be safe (it doesn't crash at runtime) and as simple as possible.

Install from *crates.io*:

```sh
cargo install mers
```

## what makes it special

### Simplicity

Mers is simple. There are only a few expressions:

- Values (`1`, `-2.4`, `"my string"`)
- Blocks (`{ statement1, statement2 }` - comma is optional, newline preferred)
- Tuples (`(5, 2)`) and Objects (`{ x: 5, y: 2 }`)
- Variable initializations (`:=`)
- Assignments (`=`)
- Variables: `my_var`, `&my_var`
- If statements: `if <condition> <then> [else <else>]`
- Functions: `arg -> <do something>`
- Function calls: `arg.function`
  + or `arg1.function(arg2, arg3)` (nicer syntax for `(arg1, arg2, arg3).function`)
- Type annotations: `[Int] (1, 2, 3).sum`
  + Type definitions: `[[MyType] Int]`, `[[TypeOfMyVar] := my_var]`

Everything else is implemented as a function.

### Types and Safety

Mers is built around a type-system where a value could be one of multiple types.

```
x := if condition { 12 } else { "something went wrong" }
```

In mers, the compiler tracks all the types in your program,
and it will catch every possible crash before the program even runs:
If we tried to use `x` as an int, the compiler would complain since it might be a string, so this **does not compile**:

```
list := (1, 2, if true 3 else "not an int")
list.sum.println
```

Type-safety for functions is different from what you might expect.
You don't need to tell mers what type your function's argument has - you just use it however you want as if mers was a dynamically typed language:

```
sum_doubled := iter -> {
  one := iter.sum
  (one, one).sum
}
(1, 2, 3).sum_doubled.println
```

We could try to use the function improperly by passing a string instead of an int:

```
(1, 2, "3").sum_doubled.println
```

But mers will catch this and show an error, because the call to `sum` inside of `sum_doubled` would fail.

#### Type Annotations

Eventually, mers' type-checking will cause a situation where calling `f1` causes an error,
because it calls `f2`, which calls `f3`,
which tries to call `f4` or `f5`, but neither of these calls are type-safe,
because, within `f4`, ..., and within `f5`, ..., and so on.

Error like this are basically unreadable, but they can happen.

To prevent this, we should type-check our functions (at least the non-trivial ones) to make sure they do what we want them to do:

```
f1 := /* ... */
// Calling `f1` with an int should cause it to return a string.
// If `f1` can't be called with an int or it doesn't return a string, the compiler gives us an error here.
[(Int -> String)] f1
```

We can still try calling `f1` with non-int arguments, and it may still be type-safe and work perfectly fine.
If you want to deny any non-int arguments, the type annotation has to be in `f1`'s declaration or you have to redeclare `f1`:

```
f1 := [(Int -> String)] /* ... */
// or
f1 := /* ... */
f1 := [(Int -> String)] f1
```

This hard-limits the type of `f1`, similar to what you would expect from functions in statically typed programming languages.

However, even when using type annotations for functions, mers can be more dynamic than most other languages:

```
f1 := [(Int/Float -> String, String -> ()/Float)] /* ... */
```

Here, `f1`'s return type depends on the argument's type: For numbers, `f1` returns a string, and for strings, `f1` returns an empty tuple or a float.
Of course, if `f1`'s implementation doesn't satisfy these requirements, we get an error.

### Error Handling

Errors in mers are normal values.
For example, `("ls", ("/")).run_command` has the return type `({Int/Bool}, String, String)/RunCommandError`.
This means it either returns the result of the command (exit code, stdout, stderr) or an error (a value of type `RunCommandError`).

So, if we want to print the programs stdout, we could try

```
(s, stdout, stderr) := ("ls", ("/")).run_command
stdout.println
```

But if we encountered a `RunCommandError`, mers wouldn't be able to assign the value to `(s, stdout, stderr)`, so this doesn't compile.
Instead, we need to handle the error case, using the `try` function:

```
("ls", ("/")).run_command.try((
  (s, stdout, stderr) -> stdout.println,
  error -> error.println,
))
```

For your own errors, you could use an object: `{err: { read_file_err: { path: /* ... */, reason: /* ... */ } } }`.
This makes it clear that the value represents an error and it is convenient when pattern-matching:

- `{ err: _ }`: all errors
- `{ err: { read_file_err: _ } }`: only read-file errors
- `{ err: { parse_err: _ } }`: only parse errors
- `{ err: { read_file_err: { path: _, reason: { permission_denied: _ } } } }`: only read-file: permission-denied errors
- ...
