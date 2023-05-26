# Hello, world!

Welcome to mers! Start by creating a file for your mers code:

    println("Hello, world!")

now run it: `mers <file>` (replace mers the location of the compiled executable if it's not in your PATH)

# The basics

To write a comment, type `//`. From here until the end of the line, whatever you write will be ignored completely.
To write a comment that spans multiple lines or ends before the end of the line, put your text between `/*` and `*/`.

    if 10 != 15 /* pretend this actually meant something... */ {
        // does something
    } else {
        // TODO: implement this!
    }

To declare a new variable, `:=` is used.

    println("Hello! Please enter your name.")
    username := read_line()
    println("Hello, " + username)

To change an existing variable, prefix it with `&` and use `=` instead of `:=`.

    username := "<unknown>"
    println("Hello! Please enter your name.")
    &username = read_line()
    println("Hello, " + username)

To call a function, write `function_name(argument1, argument2, argument3, ...)` with as many arguments as you need:

    add(10, 25)

To avoid too many nested function calls, you can also put the first argument in front of the function and have a dot behind it:

    10.add(25)

The following two lines are identical:

    println(to_string(add(10, 25)))
    10.add(25).to_string().println()

# Conditions - If/Else

An `if`-statement is used to only do something if a condition is met.
In mers, no brackets are necessary to do this.
You can just write `if <any condition> <what to do>`.

    println("Hello! Please enter your name.")
    username := read_line()
    if username.len() > 0 {
        println("Hello, " + username)
    }

You may have noticed that the `{` and `}` shown in the example weren't mentioned before.
That's because they are totally optional! If you only want to do one thing, like the `println` in our case, you can leave them out. However, this can get messy very quickly.

    println("Hello! Please enter your name.")
    username := read_line()
    if username.len() > 0
        println("Hello, " + username)

The indentation here also isn't required - mers doesn't care about indentation. A single space is as good as any number of spaces, tabs, or newline characters.

By adding `else <what to do>` after an if statement, we can define what should happen if the condition isn't met:

    println("Hello! Please enter your name.")
    username := read_line()
    if username.len() > 0 {
        println("Hello, " + username)
    } else {
        println("Hello!")
    }

# Loops - Loop

Let's force the user to give us their name.

For this, we'll do what any not-insane person would do: Ask them repeatedly until they listen to us.

    username := ""
    loop {
        println("Hello! Please enter your name.")
        &username = read_line()
        if username.len() > 0 {
            true
        }
    }
    println("Hello, " + username)

The most obvious thing that stands out here is `loop`. This just repeatedly runs its statement.
Since mers doesn't have `break`s or `return`s, we need a different way to exit the loop.
This is done by returning `true` from the block, which is exactly what the if statement is doing.

We can simplify this by just removing the `if` entirely - if `username.len() > 0` returns `true`, we break from the loop, if it returns `false` we don't.

    username := ""
    loop {
        println("Hello! Please enter your name.")
        &username = read_line()
        username.len() > 0
    }
    println("Hello, " + username)

There is one more thing here that I would like to change: We assign a default value to `username`, in this case an empty string `""`.
Default values like this one can lead to all kinds of unexpected behavior and, in my opinion, should be avoided whenever possible.
Luckily, mers' `loop` can actually return something - specifically, it returns the `match` of the inner statement, if there is one.
Sounds quite abstract, but is very simple once you understand matching.

# Matching

In mers, some values match and some don't.
This is often used for advanced conditions, but we also need it to break from a loop.

- Values that don't match are an empty tuple `[]`, `false`, and enums.
- Tuples of length 1 match with the value contained in them: `[4]` becomes `4`.
- Other values match and don't change: `true` becomes `true`, `"some text"` becomes `"some text"`.

# Loops... again

The actual reason the loop stops once we return `true` is because `true` is a value that matches.
We didn't do anything with it, but the loop actually returned `true` in the previous example.
Since the value we want to get out of the loop is the `username`, not just `true`, we have to return the username from the loop:

    username := loop {
        println("Hello! Please enter your name.")
        input := read_line()
        if input.len() > 0 {
            input
        }
    }
    println("Hello, " + username)

If the input isn't empty, we return it from the loop since a value of type `string` will match. The value is then assigned to `username` and printed.

# Match statements

Match statements let you define multiple conditions in a row.
Their superpower is that they use matching, while `if` statements just use `bool`s (`true` and `false`).
One of my favorite examples for mers' strength is this:

    input := read_line()
    number := match {
        input.parse_int() n n
        input.parse_float() n n
        [true] [] []
    }
    number.debug()

Unfortunately, this needs quite a lengthy explanation.

First, `parse_int()` and `parse_float()`. These are functions that take a string as their argument and return `[]/int` or `[]/float`.
They try to read a number from the text and return it. If this fails, they return `[]`.

Conveniently (except not - this is obviously on purpose), `[]` doesn't match while `int` and `float` values do.

This is where the magic happens: the `match` statement.
Between the `{` and the `}`, you can put as many "match arms" as you want.
Each of these arms consists of three statements: the condition, something to assign the value to, and an action.

Let's look at the match arm `input.parse_int() n n`.
The three statements here are `input.parse_int()` (condition), `n` (assign_to), and `n` (action).

If the input isn't a number, `input.parse_int()` will return `[]`. Since this doesn't match, the second match arm (`input.parse_float()`) will try to parse it to a float instead.

If the input is a number, `input.parse_int()` will return an `int`. Since this matches, the match arm will be executed.
First, the matched value - the `int` - will be assigned to `n`. the assign_to part behaves like the left side of a `:=` expression, with the matched `int` in the right.
Finally, the action statement uses our new variable `n` which contains the number we have parsed and returns it from the match statement.

Since the two arms in the match statement return `int` and `float`, the match statement will return `int/float/[]`.
To get rid of the `[]`, we need to add a third arm: `[true] [] "default value"`. `[true]` is a value that the compiler knows will always match - a tuple of length 1. Assigning something to an empty tuple `[]` just gets rid of the value.
The return type is now `int/float/string`.

Finally, we `debug()` the variable. Debug is a builtin function that prints the expected type (statically determined at compile-time), the actual type, and the value.
If we input `12.3`, it outputs `int/float/[] :: float :: 12.3`.
If we input `9`, it outputs `int/float/[] :: int :: 9`.

# Switch statements

As demonstrated in the previous example, variables (and all values) in mers can have a type that actually represents a list of possible types.
If you wish to filter the types, you can use the `switch` statement.

For example, here we only print "Number: _" if x is actually a number.

    x := if true 10 else "text"
    switch x {
        int num println("Number: " + num.to_string())
    }

In most cases, you should use `switch!` instead of `switch`.
This simply forces you to handle all possible types `x` could have:

    x := if true 10 else "text"
    switch! x {
        int num println("Number: " + num.to_string())
    }

After adding the `!`, mers will refuse to compile:

> Switch! statement, but not all types covered. Types to cover: string    

To fix this, we have to cover the `string` type:


    x := if true 10 else "text"
    switch! x {
        int num println("Number: " + num.to_string())
        string s println("String: " + s)
    }

We have now covered every possible type `x` can have, and mers happily accepts the `!`.
By adding the `!`, mers will not add `[]` to the switch statement's output type, since one of the arms will always be executed and provide some value that eliminates the need for a `[]` fallback.

# Loops - For

Mers also has a for loop. The syntax for it is `for <assign_to> <iterator> <what_to_do>`, for example `for number [1, 2, 4, 8, 16, ...] { println("Number: " + number.to_string()) }`.
The `...]` indicates that this is a list instead of a tuple. In this case, it doesn't make a difference, but lists are more common in for loops which is why this is what the example uses.

Just like normal `loop`s, the for loop will exit if `<what_to_do>` returns a value that matches.

If you want custom iterators, all you need is a function that takes no arguments and returns any value. If the returned value matches, it is assigned and the loop will run. If it doesn't match, the loop will exit.

# END

The best way to learn about mers is to use it. If you get stuck, a look at the examples or the syntax cheat sheet may help. Good luck, have fun!
