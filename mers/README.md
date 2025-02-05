# mers

Mers is a simple, safe programming language.

```sh
cargo install mers
```

# examples

```
"Hello, World!".println
```

In mers, `.function` is the syntax used to call functions.
Everything before the `.` is the function's argument.
In this case, our argument is the string containing `Hello, World!`.

---

```
greeting := "Hello, World!"
greeting.println
```

We use `name := value` to declare a variable, in this case `my_var`.
We can then simply write `my_var` whenever we want to use its value.

---

```
say_hello := () -> "Hello, World!".println
().say_hello
```

We create a function using the `->` syntax, then assign it
to the `say_hello` variable.
We then call the function with the `()` argument.

# safety & type system

Mers is type-checked, which guarantees
that a valid mers program will not crash
unless `exit` or `panic` is called.

The type system is kept simple on purpose.
A variable's type is decided where it is declared,
there is no type inference. Each type of expression
has a defined way of finding the type of values it
produces, for example:

- tuples and objects produce tuple and object types
- a block produces the same type of value as the last expression it contains
- an if-statement produces either the type of the first expression (if the condition was true), or the type of the second expression (which is `()` if there is no `else`)
- type hint expressions produce the type specified in square brackets
- ...

Mers can represent sum- and product-types:

- product types are tuples or objects: `(A, B)` or `{ a: A, b: B }`
- sum types are just two types mixed together: `A/B`

An example of product types:

```
// this is an Int
some_number := 5
// these are Strings
some_string := "five"
some_number_as_string := (some_number).concat
// this is a { base10: String, human: String }
some_object := { base10: some_number_as_string, human: some_string }
// this is a (Int, { base10: String, human: String })
some_tuple := (some_number, some_object)
```

An example of a sum type:

```
some_number := 5
some_string := "five"
some_number_as_string := (some_number).concat
// this is an Int/String
some_value := if some_string.eq(some_number_as_string) { some_number } else { some_string }
```

# simplicity

mers only has a few different expressions:

- literals: `4`, `-1.5`, `"hello"`
- tuples and objects: `(a, b, c)`, `{ a: 1, b: 2 }`
- variable declarations: `var :=`
- variables: `var` (get the value) or `&var` (get a reference to the value)
- reference assignments: `ref =` (usually used as `&var =`)
- blocks: `{ a, b, c }`
- functions: `arg -> expression`
- function calls: `arg.func` or `a.func(b, c)`, which becomes `(a, b, c).func`
- `if condition expression` and `if condition expression_1 else expression_2`
- `loop expression`
- type hints `[Int] 5`
- type definitions `[[Number] Int/Float]` or `[[TypeOfX] := x]`, which can also be used as a type check: `[[_] := expression]` checks that the expression is type-correct
- try: mers' switch/match: `x.try(num -> num.div(2), _ -> 0)`

mers treats everything as call-by-value by default:

```
modify := list -> {
  &list.insert(1, "new value")
  list.debug
}

list := ("a", "b").as_list
list.modify
list.debug
```

When `modify` is called, it changes its copy of `list` to be `[a, new value, b]`.
But when `modify` is done, the original `list` is still `[a, b]`.

If you wanted list to be changed, you would have return the new list

```
modify := list -> {
  &list.insert(1, "new value")
  list.debug
}

list := ("a", "b").as_list
&list = list.modify
list.debug
```

or give `modify` a reference to your list

```
modify := list -> {
  list.insert(1, "new value")
  list.deref.debug
}

list := ("a", "b").as_list
&list.modify
list.debug
```

<small>To make this slightly less inefficient, mers
uses a copy-on-write system, so that you
can give copies of large values to functions
without copying the entire value.
When a copy of a value is changed, it is (at
least partially) copied before mers changes it.</small>

# more examples

```
if "a".eq("b") {
  "what?".println
}

response := if "a".eq("b") {
  "what?"
} else {
  "ok :)"
}
response.println
```

An `if` is used to conditionally execute code.
It can also produce values.

---

```
val := loop {
  "> ".print
  ().read_line.trim.parse_float
}
val.println
```

This program asks the user for a number.
If they type a valid number, it prints that number.
If they don't type a valid number, they will be asked again.
This works because `parse_float` returns `()/(Float)`, which happens to align with how loops in mers work: \
A `loop` will execute the code. If it is `()`, it will execute it again. If it is `(v)`, the loop stops and returns `v`.

---

```
val := if "a".eq("a") {
  5
} else {
  "five"
}
val.try(
  // if the value is a number, print half of it
  num -> num.div(2).println
  // for any other value, print it directly
  other -> other.println
)
```

A `try` expression uses the first type-correct branch for the given value.
In this case, for a number, we can do `num.div(2)`, so the first branch is taken.
For non-number values, `.div(2)` would be a type error, so the second branch has to be taken.

---

```
add_one := x -> x.add(1)
do_twice := func -> x -> x.func.func
add_two := add_one.do_twice
2.add_two
```
