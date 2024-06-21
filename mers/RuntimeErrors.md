# Runtime Errors in Mers

This is a list of <small><small><small><small>(hopefully)</small></small></small></small>
all runtime errors that can occur when running a mers program.

## Explicit

- Calling `panic` with a `String`-type argument will cause a runtime error, using the argument as the error message.
- Calling `exit` will exit with the given (`Int`) exit code. Usually, a nonzero exit code is considered a program failure. While this isn't really a runtime error, `exit` terminate the program just like an error would.

## Integer-Integer math

Some math functions fail under certain conditions when called with two integer arguments:

- `x.div(0)` will fail because you cannot divide by zero. If at least one argument is a `Float`, this will return Infinity or Not A Number.
- `x.modulo(0)` will fail
- ...

## ... (TODO)
