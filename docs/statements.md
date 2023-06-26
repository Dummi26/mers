# Mers statements overview

This is the documentation for mers statements.
In code, statements are represented by `SStatement`, `SStatementEnum`, `RStatement`, and `RStatementEnum`.

## Statement prefixes

A statement can be prefixed with any number of stars `*`. This is called dereferencing and turns a reference to a value into the value itself. Modifying the value after a dereference leaves the value the reference was pointing to untouched (the data will be cloned to achieve this).

A statement can also be prefixed with an arrow followed by the type the statement will have: `-> int/float 12`.
Although the statement will always return an `int`, mers will assume that it could also return a float.
If the statement's output doesn't fit in the forced type (`-> int "text"`), mers will generate an error.
In combination with functions, this is similar to rust's syntax:

    fn my_func(a int, b int) -> int {
        a + b
    }

## Assignment

Statements can assign their value instead of returning it.
The syntax for this is `<ref_statement> = <statement>` or `<assign_to> := <statement>`.

If just `=` is used, the left side must return a reference to some value which will then be changed to the value generated on the right.
If `:=` is used, new variables can be declared on the left.

Destructuring is possible too: `[a, b] := [12, 15]`.

# Different statements

## Value

A statement that always returns a certain value: `10`, `"Hello, World"`, ...

## Code block

Code blocks are a single statement, but they can contain any number of statements.
They have their own scope, so local variables declared in them aren't accessible from outside.

They return whatever the last statement they contain returns. Empty code blocks (ones without any statements: `{}`) return `[]`.

    list := [1, 2, 3, 4 ...]
    sum := {
      // a temporary variable that only exists in this block
      counter := 0
      for elem list {
        &counter = counter + elem
      }
      // the final value will be returned from the block and then assigned to sum.
      counter
    }

## Tuple and list

These statements contain any number of statements and return the returned values in order as a tuple or list.

    // x is a tuple and has type [int string float]
    x := [
      get_num(),
      get_text(),
      12.5
    ]
    // y is a list and has type [int/string/float ...]
    x := [
      get_num(),
      get_text(),
      12.5
    ...]

## Variable

These statements retrieve the value from a variable name:

    x := "my value"
    println(
      x // <- this is a variable statement - it gets the "my value" from above so the println function can use it.
    )

They can also get references if the variable name is prefixed with the `&` symbol:

    debug(&x) // type is &string

This is why `&` is required to assign to a variable:

    x = "something else" // can't assign to string!
    &x = "something else" // only data behind references can be changed - we can assign to &string.

## Function call

Function calls give some data to a function and return the result from that function.

There are 3 kinds of function calls:

- calls to builtin functions
- calls to library functions
- calls to custom functions

Anonymous functions are called using the builtin `run` function.

## If statement

Allow for conditional execution of code paths:

    if condition() {
      // do something
    }

    if condition() {
      // do something
    } else {
      // do something else
    }

## Loop

Executes code repeatedly:

    counter := 0
    loop {
      &counter = counter + 1
      println("c: " + counter.to_string())
    }

## For loop

## Switch

## Match

## Enum

Wrap a value in an enum: `EnumName: <statement>`

# dot-chains

Statements can be followed by the dot `.` character. This has different meanings depending on what comes after the dot:

## A function call

In this case, the statement before the dot is prepended to the function arguments:

    a.func(b, c)
    func(a, b, c)

## An integer

Gets the n-th thing from a tuple (uses zero-based indexing):

    [1, "2", 3.0].1 == "2"

If the value before the dot is a reference to a tuple, a reference to the thing inside the tuple will be obtained:

    x := [1, "2", 3.0]
    // we assign to the "2", which changes it.
    &x.1 = "4"
