# mers

Mers is a high-level programming language.
It is designed to be safe (it doesn't crash at runtime) and as simple as possible.

## what makes it special

### Simplicity

Mers aims to be very simple, as in, having very few "special" things.
But this means that many things you may be familiar with simply don't exist in mers,
because they aren't actually needed.
This includes:

**Exceptions**, because errors in mers are just values.
A function to read a UTF-8 text file from disk could have a return type like `String/IOError/UTF8DecodeError`,
which tells you exactly what errors can happen and also forces you to handle them (see later for mers' type-system).

**Loops**, because, as it turns out, a function which takes an iterable and a function can do this just fine:
```
my_list := (1, 2, 3, "four", "five", 6.5).as_list
(my_list, val -> val.println).for_each
```
Javascript has `forEach`, Rust `for_each`, C# `Each`, and Java has `forEach`.
At first, it seemed stupid to have a function that does the same thing a `for` or `foreach` loop can already do,
but if a function can do the job, why do we even need a special language construct to do this for us?
To keep it simple, mers doesn't have any loops - just functions you can use to loop.

**Breaks** aren't necessary, since this can be achieved using iterators or by returning `(T)` in a `loop`.
It is also similar to `goto`, because it makes control flow less obvious, so it had to go.

The same control flow obfuscation issue exists for **returns**, so these also aren't a thing.
A function simply returns the value created by its expression.

**Functions** do exist, but they have one key difference: They take exactly one argument. Always.
Why? Because we don't need anything else.
A function with no arguments now takes an empty tuple `()`,
a function with two arguments now takes a two-length tuple `(a, b)`,
a function with either zero, one, or three arguments now takes a `()/(a)/(a, b, c)`,
and a function with n arguments takes a list, or a list as part of a tuple, or an optional list via `()/<the list>`.

### Types and Safety

Mers is built around a type-system where a value could be one of multiple types.
This is basically what dynamic typing allows you to do:
```
x := if condition { 12 } else { "something went wrong" }
```
This would be valid code.
However, in mers, the compiler still tracks all the types in your program,
and it will catch every possible crash before the program even runs:
If we tried to use `x` as an int, the compiler would complain since it might be a string, so this does not compile:
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

(note: type-checks aren't implemented for all functions yet, you may need to use `--check no` to get around this and deal with runtime panics for now)

## docs

docs will be available some time. for now, check mers_lib/src/program/configs/*
