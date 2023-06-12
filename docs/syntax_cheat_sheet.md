# mers syntax cheat sheet

## Types

- `bool`
- `int`
- `float`
- `string`
- tuple: `[<type1> <type2> <type3> <...>]`
- list: `[<type> ...]`
- function: `fn(<input-output-map>)` (might change? depends on implementation of generics)
- thread: `thread(<return_type>)`
- reference: `&<type>` or `&(<type1>/<type2>/<...>)`
- enum: `EnumName(<type>)`
- one of multiple types: `<type1>/<type2>/<type3>`

## Values

- bool
  + `true` or `false`
- int
  + decimal: `2`, `+2`, `-5`, `0`, ...
- float
  + decimal: `1.5`, `0.5`, `-5.2`, `2.0`, ... (this is recommended for compatability and clarity)
  + whole numbers: `1.0`, `1.`, ... (may break with future updates)
  + numbers from 0 to 1: `.5` would be ambiguous following tuples, so is not supported.
- string
  + `"my string"` (double quotes)
  + `"it's called \"<insert name>\""` (escape inner double quotes with backslash)
  + all escape sequences
    * `\"` double quote character
    * `\\` backslash character
    * `\n` newline
    * `\r` carriage return
    * `\t` tab
    * `\0` null
- tuple
  + `[<val1> <val2> <val3> <...>]`
- list
  + `[<val1> <val2> <val3> <...> ...]` (like tuple, but `...]` instead of `]`)
- function
  + `(<arg1> <arg2> <arg3> <...>) <statement>` where `<argn>` is `<namen> <typen>`.
- thread
  + returned by the builtin function `thread()`
- reference
  + to a variable: `&<varname>`
  + to something else: usually using `get()` or its equivalent on a reference to a container instead of the container itself: `&list.get()` instead of `list.get()`
- enum
  + `<EnumName>: <statement>`

## Variables

- declaration and initialization
  + `<var_name> := <statement>`
  + can shadow previous variables with the same name: `x := 5 { x := 10 debug(x) } debug(x)` prints `10` and then `5`.
- assignment
  + `&<var_name> = <statement>`
    * modifies the value: `x := 5 { &x = 10 debug(x) } debug(x)` prints `10` and then `10`.
  + `<statement_left> = <statement_right>`
    * assigns the value returned by `<statement_right>` to the value behind the reference returned by `<statement_left>`.
    * if `<statement_right>` returns `<type>`, `<statement_left>` has to return `&<type>`.
    * this is why `&<var_name> = <statement>` is the way it is.
  + `***<statement_left> = <statement_right>`
    * same as before, but performs dereferences: `&&&&int` becomes `&int` (minus 3 references because 3 `*`s), so a value of type `int` can be assigned to it.
- destructuring
  + values can be destructured into tuples or lists (as of right now):
  + `[a, b] := [10, "some text"]` is effectively the same as `a := 10, b := "some text"`

## Statements

- value
  + `10`, `true`, `"text"`, `"[]"`, etc
- tuple
  + `[<statement1> <statement2> <...>]`
- list
  + `[<statement1> <statement2> <...> ...]`
- variable
  + `<var_name>` (if the name of the variable isn't a value or some other kind of statement)
- function call
  + `<fn_name>(<arg1> <arg2> <...>)`
  + `<fn_name>(<arg1>, <arg2>, <...>)`
  + `<arg1>.<fn_name>(<arg2>, <...>)`
- function definition
  + `fn <fn_name>(<arg1> <arg2> <...>) <statement>` where `<argn>` is `<namen> <typen>`
- block
  + `{ <statement1> <statement2> <...> }`
- if
  + `if <condition> <statement>` (`if true println("test")`)
  + `if <condition> <statement> else <statement>` (`if false println("test") else println("value was false")`)
- loop
  + `loop <statement>`
  + if the statement returns a value that matches, the loop will end and return the matched value
  + the loop's return type is the match of the return type of the statement
- for loop
  + `for <assign_to> <iterator> <statement>`
  + in each iteration, the variable will be initialized with a value from the iterator.
  + iterators can be lists, tuples, or functions.
  + for function iterators, as long as the returned value matches, the matched value will be used. if the value doesn't match, the loop ends.
  + if the statement returns a value that matches, the loop will end and return the matched value
  + the loop's return type is the match of the return type of the statement or `[]` (if the loop ends because the iterator ended)
- switch
  + `switch <value> { <arm1> <arm2> <...> }`
    * where `<armn>` is `<typen> <assign_to> <statementn>`
    * if the variable's value is of type `<typen>`, `<statementn>` will be executed with `<value>` assigned to `<assign_to>`.
    * if the variables type is included multiple times, only the first match will be executed.
    * within the statement of an arm, the variables type is that specified in `<typen>`, and not its original type (which may be too broad to work with)
  + `switch! <var_name> { <arm1> <arm2> <...> }`
    * same as above, but all types the variable could have must be covered
    * the additional `[]` in the return type isn't added here since it is impossible not to run the statement of one of the arms.
- match
  + `match { <arm1> <arm2> <...> }`
    * where `<armn>` is `<(condition) statement> <assign_to> <(action) statement>`
    * each arm has a condition statement, something that the matched value will be assigned to, and an action statement.
    * if the value returned by the condition statement matches, the matched value is assigned to `<assign_to>` and the action statement is executed.
    * only the first matching arm will be executed. if no arm was executed, `[]` is returned.
- fixed-indexing
  + `<statement>.n` where `n` is a fixed number (not a variable, just `0`, `1`, `2`, ...)
  + `<statement>` must return a tuple or a reference to one. `<statement>.n` then refers to the nth value in that tuple.
  + for references to a tuple, references to the inner values are returned.
- enum
  + `<EnumName>: <statement>`
- type definition
  + `type <name> <type>` (`type Message [string string]`)
- macros
  + `!(<macro_type> <...>)`
  + `!(mers { <code> })`
    * compiles and runs the code at compile-time, then returns the computed value at runtime.
  + `!(mers <file_path>)` or `!(mers "<file path with spaces and \" double quotes>")`
    * same as above, but reads code from a file instead
    * path can be relative

## Matching

- values that don't match
  + `[]`
  + `false`
- values that match
  + `[v]` -> `v`
  + `v` -> `v` unless otherwise specified
- invalid
  + `[v1 v2]` or any tuple whose length isn't `0` or `1`
