# mers

Mers is a high-level programming language.
It is designed to be safe (it doesn't crash at runtime) and as simple as possible.

See also:
[Quickstart](Quickstart.md),

## what makes it special

### Simplicity

Mers is simple. There are only few expressions:

- Values (`1`, `"my string"`, ...)
- Blocks (`{}`)
- Tuples (`()`)
- Assignments (`=`)
- Variable initializations (`:=`)
- Variables (`my_var`, `&my_var`)
- If statements (`if <condition> <then> [else <else>]`)
- Functions (`arg -> <do something>`)
- Function calls `arg.function`

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

### Error Handling

Errors in mers are normal values.
For example, `("ls", ("/")).run_command` has the return type `({Int/()}, String, String)/RunCommandError`.
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

## docs

docs will be available in some time. for now, check mers_lib/src/program/configs/*
