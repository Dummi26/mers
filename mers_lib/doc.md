# mers documentation

## ISSUES

when storing a reference, then reinitializing a variable of the same name, the reference may get the new value although
it would be expected for it to be a reference to the value before it was reinitialized.

## parsing

syntax:

- `// <comment>`
- `/* <comment> */`

operators:
- `<target> := <source>` init
- `<target> = <source>` assign (`<target>` must be a reference)
- `+`
- `-`
- `*`
- `/`
- `%`
- `&`
- `|`
- `&&`
- `||`

keywords (must be between whitespace):

- `if <condition> <statement>`
- `else <statement>` (after `if`)
- `loop <statement>`
- `switch { <arms> }`
- `<arg> -> <statement>`
- `def <name> <: for types, = for comptime> <_>` for compile-time stuff (types, macros, ...)

## details

### functions

A function takes an argument and returns some data:

    func := input -> input + 2
    3.func.println // 5
    (list, 0).get // first element
    (val, match -> match.println, [] -> "doesn't match".println).match

### switch

    switch <val> {
        <type> <func>
    }

    switch something {
        int num -> {"int: " + num}.println
        float num -> {"float: " + num}.println
    }

