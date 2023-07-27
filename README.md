# mers

Mers is getting a rewrite!

This means that this README isn't complete,
many things will change,
and the docs/ are for a completely different language.

## why mers?

### Simplicity

Mers aims to be very simple, as in, having very few "special" things.
But this means that many things you may be familiar with simply don't exist in mers,
because they aren't actually needed.
This includes:

**Exceptions**, because errors in mers are values just like any other.
A function to read a UTF-8 text file from disk could have a return type like `String/IOError/UTF8DecodeError`,
which tells you exactly what errors can happen and also forces you to handle them (see later for mers' type-system).

**Loops**, because, as it turns out, a function which takes an iterable and a function can do this just fine.
Javascript has `forEach`, Rust `for_each`, C# `Each`, and Java has `forEach`.
At first, it seemed stupid to have a function that does the same thing a `for` or `foreach` loop can already do,
but if a function can do the job, why do we even need a special language construct to do this for us?
To keep it simple, mers doesn't have any loops except for `loop`.
`loop` simply repeats until the inner expression returns `(T)`, which causes loop to return `T`.

**Breaks** aren't necessary, since this can be achieved using iterator magic or by returning `(T)` in a  `loop`.
It is also similar to `goto`, because it makes control flow less obvious, so it had to go.

The same control flow obfuscation issue exists for **returns**, so these also aren't a thing.
A function simply returns the value created by its expression.

**Functions** do exist, but they have one key difference: They take exactly one argument. Always.
Why? Because we don't need anything else.
A function with no arguments now takes an empty tuple `()`,
a function with two arguments now takes a two-length tuple `(a, b)`,
a function with either zero, one, or three arguments now takes a `()/(a)/(a, b, c)`,
and a function with n args takes a list, or a list as part of a tuple, or an optional list via `()/<the list>`.

### Types

Mers is built around a type-system where a value could be one of multiple types.
This is basically what dynamic typing allows you to do:

    x := if condition { 12 } else { "something went wrong" }

This would be valid code.
However, in mers, the compiler still tracks all the types in your program,
and it will catch every possible crash before the program even runs:
If we tried to use `x` as an Int, the compiler would complain since it might be a string.

(note: type-checks aren't implemented since the rewrite is just barely functional, but they will be there and fully working soon)
