# mers

```sh
cargo install mers
```

Mers is a simple, safe programming language.

## features

- mers' syntax is simple and concise
- mers is type-checked, but behaves almost like a dynamically typed language
- it has no nulls or exceptions
- references in mers are explicit: `&var` vs. just `var`
- no `goto`s (or `break`s or `return`s)
- locking (useful for multithreading, any reference can be locked)

# examples

## Hello, World!

![image](https://github.com/Dummi26/mers/assets/67615357/f9771400-f450-41dd-95d6-05560259ad44)

In mers, `.function` is the syntax used to call functions.
Everything before the `.` is the function's argument.
In this case, our argument is the string containing *Hello, World!*,

## Variables

![image](https://github.com/Dummi26/mers/assets/67615357/7b603b1f-6a74-4e48-8673-b91cdaf49095)

We use `name := value` to declare a variable, in this case `my_var`.
We can then simply write `my_var` whenever we want to use its value.

## If

![image](https://github.com/Dummi26/mers/assets/67615357/64956ed7-b206-4e0b-8bca-f5310498a4e9)

An `if` is used to conditionally execute code.
Obviously, since our condition is always `true`, our code will always run.

The condition in an `if` has to be a bool, otherwise...

![image](https://github.com/Dummi26/mers/assets/67615357/95c598b7-f1ce-41cd-9dbe-1709e2d0d5b9)

## Else

![image](https://github.com/Dummi26/mers/assets/67615357/7dfae822-a2af-4920-9be7-54b9d92af4b4)

We can add `else` directly after an `if`. This is the code that will run if the condition was `false`.

## Using If-Else to produce a value

Depending on the languages you're used to, you may want to write something like this:

```js
var result
if (condition) {
  result = "Yay"
} else {
  result = "Nay"
}
```

But in mers, an `if-else` can easily produce a value:

![image](https://github.com/Dummi26/mers/assets/67615357/af698141-0c5f-49c1-bf45-1732eb7633c4)

We can shorten this even more by writing

![image](https://github.com/Dummi26/mers/assets/67615357/053b8887-fc42-4fe1-93be-d8d5d2a84192)

## What if the branches don't have the same type?

Rust also allows us to return a value through `if-else` constructs, as long as they are of the same type:

```rs
if true {
  "Yep"
} else {
  "Nay"
}
```

But as soon as we mix two different types, it no longer compiles:

```rs
if true {
  "Yep"
} else {
  5 // Error!
}
```

In mers, this isn't an issue:

![image](https://github.com/Dummi26/mers/assets/67615357/40988b0e-b692-413c-a4d7-1675c90e9662)

The variable `result` is simply assigned the type `String/Int`, so mers always knows that it has to be one of those two.

We can see this if we add a type annotation:

![image](https://github.com/Dummi26/mers/assets/67615357/1047d922-17f8-4258-a2c2-360e547ab65e)

Obviously, the `if-else` doesn't always return an `Int`, which is why we get an error.

## Using If without Else to produce a value

If there is no `else` branch, mers obviously has to show an error:

![image](https://github.com/Dummi26/mers/assets/67615357/907269f3-6cb9-46d2-9f29-8ebe9e1c40ca)

Or so you thought... But no, mers doesn't care. If the condition is false, it just falls back to an empty tuple `()`:

![image](https://github.com/Dummi26/mers/assets/67615357/d30ef92c-2653-4366-bb49-04c5c69ee2c2)

## Sum of numbers

![image](https://github.com/Dummi26/mers/assets/67615357/1f988597-7aca-4d77-bac8-57b99445b7f7)

## Sum of something else?

If not all of the elements in our `numbers` tuple are actually numbers, this won't work.
Instead, we'll get a type-error:

![image](https://github.com/Dummi26/mers/assets/67615357/ef8f14a9-5e45-48f4-bb66-3806bc642ba5)

## Loops

![image](https://github.com/Dummi26/mers/assets/67615357/784ea761-f98d-459a-93cf-d00b076a955b)

This program asks the user for a number. if they type a valid number, it prints that number.
If they don't type a valid number, they will be asked again.

This works because `parse_float` returns `()/(Float)`, which happens to align with how loops in `mers` work:

A `loop` will execute the code. If it is `()`, it will execute it again.
If it is `(v)`, the loop stops and returns `v`:

![image](https://github.com/Dummi26/mers/assets/67615357/271deba8-fbbb-4113-9fff-d13a557031f6)

With this, we can loop forever:

![image](https://github.com/Dummi26/mers/assets/67615357/d0b23656-4177-40bf-9f49-e69e0f535396)

We can implement a while loop:

![image](https://github.com/Dummi26/mers/assets/67615357/9e902de0-04bb-4799-ab1b-8a097574e8c7)

Or a for loop:

![image](https://github.com/Dummi26/mers/assets/67615357/bfcd5107-4f9e-4425-817e-e5df9495eb46)

The `else (())` tells mers to exit the loop and return `()` once the condition returns `false`.

## Functions

Functions are expressed as `arg -> something`, where `arg` is the function's argument and `something` is what the function should do.
It's usually convenient to assign the function to a variable so we can easily use it:

![image](https://github.com/Dummi26/mers/assets/67615357/d313c2bd-cf03-4dd4-9abd-d9d96b52c64a)

Since functions are just normal values, we can pass them to other functions, and we can return them from other functions:

![image](https://github.com/Dummi26/mers/assets/67615357/3ec15d16-5c80-4c88-b8f1-03db572674f3)

Here, `do_twice` is a function which, given a function, returns a new function which executes the original function twice.
So, `add_one.do_twice` becomes a new function which could have been written as `x -> x.add_one.add_one`.

Of course, this doesn't compromise type-safety at all:

![image](https://github.com/Dummi26/mers/assets/67615357/200a80eb-19f3-4534-b403-f47727a4da8e)

Mers tells us that we can't call `add_two` with a `String`,
because that would call the `func` defined in `do_twice` with that `String`, and that `func` is `add_one`,
which would then call `sum` with that `String` and an `Int`, which doesn't work.

The error may be a bit long, but it tells us what went wrong.
We could make it a bit more obvious by adding some type annotations to our functions:

![image](https://github.com/Dummi26/mers/assets/67615357/d883ff6e-a8c9-4ab5-849f-a98d715c2c99)

## Advanced variables

In mers, we can declare two variables with the same name:

![image](https://github.com/Dummi26/mers/assets/67615357/dcfc66f1-5ad6-43d8-805d-1011a40cb277)

As long as the second variable is in scope, we can't access the first one anymore, because they have the same name.
This is not the same as assigning a new value to x:

![image](https://github.com/Dummi26/mers/assets/67615357/f4de1132-41cc-4f72-8cf5-035e8657f5dd)

The second `x` only exists inside the scope created by the code block (`{`), so, after it ends (`}`), `x` refers to the original variable again, whose value was not changed.

To assign a new value to the original x, we have to write `&x =`:

![image](https://github.com/Dummi26/mers/assets/67615357/8efb65fd-ec16-4f3b-95e2-3752c3d2882a)

## References

Writing `&var` returns a reference to `var`.
We can then assign to that reference:

![image](https://github.com/Dummi26/mers/assets/67615357/8c6a0c53-f4f3-419a-8c82-268c3791d50e)

... or:

![image](https://github.com/Dummi26/mers/assets/67615357/ce93ef1a-dd9a-4ebf-8b2e-901d85346cf3)

We aren't actually assigning to `ref` here, we are assigning to the variable to which `ref` is a reference.
This works because the left side of an `=` doesn't have to be `&var`. As long as it returns a reference, we can assign to that reference:

This is used, for example, by the `get_mut` function:

![image](https://github.com/Dummi26/mers/assets/67615357/8dcede41-368a-4162-ae85-78ac40673c8a)

Here, we pass a reference to our list (`&list`) and the index `0` to `get_mut`.
`get_mut` then returns a `()/(&{Int/String})` - either nothing (if the index is out of bounds)
or a reference to an element of the list, an `Int/String`. If it is a reference, we can assign a new value to it, which changes the list.

## Multithreading

(...)

---

Note: all of the pictures are screenshots of Alacritty after running `clear; mers pretty-print file main.mers && echo $'\e[1;35mOutput:\e[0m' && mers run file main.mers`.
