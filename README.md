# mers

Mers is an experimental programming language inspired by high-level and scripting languages, but with error handling inspired by rust.

## Features

### Multiple Types

mers' approach to types aims to provide the simplicity of dynamic typing with the safety of static typing.

To achieve this, each 'type' is actually a list of single types that are all valid in the given context. All types must be handeled or the compiler will show an error.
To avoid having to type so much, types are inferred almost everywhere. It is enough to say `x = if condition "now it's a string" else []`. The compiler will give x the type `string/[]` automatically.

`int/float` is a type that can represent integers and floats. Personally, I just call this a number.

Since there are no classes or structs, tuples are also widely used, for examples, `[[]/int string string]` is returned by the builtin run_command() on success. It represents the exit code (if it exists), stdout and stderr.

To index this tuple, you can use `tuple.0` for the exit code and `tuple.1` and `tuple.2` for stdout and stderr.

### Compile-Checked

The compiler checks your program. It will guarantee type-safety. If a variable has type `int/float/bool`, it cannot be used in the add() function, because you can't add bools.

## Intro

### mers cli

to use mers, clone this repo and compile it using cargo. (if you don't have rustc and cargo, get it from https://rustup.rs/)

for simplicity, i will assume you have the executable in your path and it is named mers. Since this probably isn't the case, just know that `mers` can be replaced with `cargo run --release` in all of the following commands.



To run a program, just run `mers script.txt`. The file needs to be valid utf8.
If you compiled mers in debug mode, it will print a lot of debugging information.



Somewhere in mers' output, you will see a line with five '-' characters: ` - - - - -`. This is where your program starts running. The second time you see this line is where your program finished. After this, You will see how long your program took to run and its output.

### Hello, World!

Since this is likely your first time using mers, let's write a hello world program:

    println("Hello, World!")

If you're familiar with other programming languages, this is probably what you expected. Running it prints Hello, World! between the two five-dash-lines.
The `"` character starts/ends a string literal. This creates a value of type `string` which is then passed to the `println()` function, which writes the string to the programs stdout.

### Hello, World?

But what if we didn't print anything?

    "Hello, World?"

Running this should show Hello, World? as the program's output. This is because the output of a code block in mers is always the output of its last statement. Since we only have one statement, its output is the entire program's output.

### Hello, Variable!

Variables in mers don't need to be declared explicitly, you can just assign a value to any variable:

    x = "Hello, Variable!"
    println(x)
    x

Now we have our text between the five-dash-lines AND in the program output. Amazing!

### User Input

The builtin `read_line()` function reads a line from stdin. It can be used to get input from the person running your program:

    println("What is your name?")
    name = read_line()
    print("Hello, ")
    println(name)

`print()` prints the given string to stdout, but doesn't insert the linebreak `\n` character. This way, "Hello, " and the user's name will stay on the same line.

### Format

`format()` is a builtin function that takes a string (called the format string) and any number of further arguments. The pattern `{n}` anywhere in the format string will be replaced with the n-th argument, not counting the format string.

    println("What is your name?")
    name = read_line()
    println(format("Hello, {0}! How was your day?" name))

### alternative syntax for the first argument

If function(a b) is valid, you can also use a.function(b). This does exactly the same thing, because a.function(args) inserts a as the first argument, moving all other args back by one.
This can make reading `println(format(...))` statements a lot more enjoyable:

    println("Hello, {0}! How was your day?".format(name))

However, this can also be overused:

    "Hello, {0}! How was your day?".format(name).println()

I would consider this to be less readable because `println()`, which is the one and only thing this line was written to achieve, is now the last thing in the line. Readers need to read the entire line before realizing it's "just" a print statement. However, this is of course personal preference.

### A simple counter

Let's build a counter app: We start at 0. If the user types '+', we increment the counter by one. If they type '-', we decrement it. If they type anything else, we print the current count in a status message.

The first thing we will need for this is a loop:

    while {
        println("...")
    }

Running this should spam your terminal with '...'.

Now let's add a counter variable and read user input:

    counter = 0
    while {
        input = read_line()
        if input.eq("+") {
            counter = counter.add(1)
        } else if input.eq("-") {
            counter = counter.sub(1)
        } else {
            println("The counter is currently at {0}. Type + or - to change it.".format(counter))
        }
    }

mers actually doesn't have an else-if, the if statement is parsed as:

    if input.eq("+") {
        counter = counter.add(1)
    } else {
        if input.eq("-") {
            counter = counter.sub(1)
        } else {
            println("The counter is currently at {0}. Type + or - to change it.".format(counter))
        }
    }

This works because most {}s are optional in mers. The parser just parses a "statement", which *can* be a block.
A block starts at {, can contain any number of statements, and ends with a }. This is why we can use {} in if statements, function bodies, and many other locations. But if we only need to do one thing, we don't need to put it in a block.
In fact, `fn difference(a int/float b int/float) if a.gt(b) a.sub(b) else b.sub(a)` is completely valid in mers (gt = "greater than").

### Getting rid of the if-else-chain

Let's replace the if statement from before with a nice match statement!

    counter = 0
    while {
        input = read_line()
        match input {
            input.eq("+") counter = counter.add(1)
            input.eq("-") counter = counter.sub(1)
            true println("The counter is currently at {0}. Type + or - to change it.".format(counter))
        }
    }

the syntax for a match statement is always `match <variable> { <match arms> }`.

A match arm consists of a condition statement and an action statement. `input.eq("+")`, `input.eq("-")`, and `true` are condition statements.
The match statement will go through all condition statements until one matches (in this case: returns `true`), then run the action statement.
If we move the `true` match arm to the top, the other two arms will never be executed, even though they might also match.

### Breaking from loops

Loops will break if the value returned in the current iteration matches:

    i = 0
    res = while {
        i = i.add(1)
        i.gt(50)
    }
    println("res: {0}".format(res))

This will increment i until it reaches 51.
Because `51.gt(50)` returns `true`, `res` will be set to `true`.

    i = 0
    res = while {
        i = i.add(1)
        if i.gt(50) i else []
    }
    println("res: {0}".format(res))

Because a value of type int matches, we now break with "res: 51". For more complicated examples, using `[i]` instead of just `i` is recommended because `[i]` matches even if `i` doesn't.

A while loop's return type will be the matches of the inner return type.

For for loops, which can also end without a value matching, the return type is the same plus the empty tuple `[]`:

    res = for i 100 {
        if 50.sub(i).eq(5) {
            [i]
        }
    }
    switch! res {}

You have to cover `[]/int` because the condition in the loop might not match for any value from 0 to 99.

### Let's read a file!

    file = fs_read(&args.get(0).assume1("please provided a text file to read!"))
    switch! {}

Since `get()` can fail, it returns `[]/[t]` where t is the type of elements in the list. To avoid handling the `[]` case, the `assume1()` builtin takes a `[]/[t]` and returns `t`. If the value is `[]`, it will cause a crash.

We get the first argument passed to our program and read it from disk using fs_read. To run this: `mers script.txt test_file.txt`.

`switch` is used to distinguish between multiple possible types. `switch!` will cause a compiler error unless *all* types are covered.
This is useful to see what type the compiler thinks a variable has: In this case `[int]/Err(string)`.

Err(string) is a string value in an Err enum. Builtin functions use the Err enum to indicate errors because there is no concrete Error type.

After handling all errors, this is the code I came up with:

    file = fs_read(&args.get(0).assume1("please provided a text file to read!"))
    switch! file {
        [int] {
            file_as_string = bytes_to_string(file)
            contents = switch! file_as_string {
                string file_as_string
                Err([string string] ) {
                    println("File wasn't valid UTF8, using lossy conversion!")
                    file_as_string.noenum().0
                }
            }
            println(contents)
        }
        Err(string) println("Couldn't read the file: {0}".format(file.noenum()))
    }

If fs_read returns an error, the file couldn't be read.

If bytes_to_string returns a string, we just return it directly.

If bytes_to_string returns an error, the file wasn't valid UTF-8. We print a warning and use the first field on the error type,
which is a string that has been lossily converted from the bytes - any invalid sequences have been replaced with the replacement character.

We then print the string we read from the file (the `contents` variable).

### Advanced match statements

(todo)

## Advanced Info / Docs

### Matching

- An empty tuple `[]`, `false`, and any enum member `any_enum_here(any_value)` will not match.
- A one-length tuple `[v]` will match with `v`, `true` will match with `true`.
- A tuple with len >= 2 is considered invalid: It cannot be used for matching because it might lead to accidental matches and could cause confusion. If you try to use this, you will get an error from the compiler.
- Anything else will match with itself.

### Single Types

Mers has the following builtin types:

- bool
- int
- float
- string
- tuple
  + the tuple type is written as any number of types separated by whitespace(s), enclosed in square brackets: [int string].
  + tuple values are created by putting any number of statements in square brackets: ["hello" "world" 12 -0.2 false].
- list
  + list types are written as a single type enclosed in square brackets: [string]. TODO! this will likely change to [string ...] or something similar to allow 1-long tuples in function args.
  + list values are created by putting any number of statements in square brackets, prefixing the closing bracket with ...: ["hello" "mers" "world" ...].
- functions
  + function types are written as fn(args) out_type (TODO! implement this)
  + function values are created using the (<first_arg_name> <first_arg_type> <second_arg_name> <second_arg_type>) <statement> syntax: anonymous_power_function = (a int b int) a.pow(b).
  + to run anonymous functions, use the run() builtin: anonymous_power_function.run(4 2) evaluates to 16.
  + note: functions are defined using the fn <name>(<args>) <statement> syntax and are different from anonymous functions because they aren't values and can be run directory: fn power_function(a int b int) a.pow(b) => power_function(4 2) => 16
- thread
  + a special type returned by the thread builtin. It is similar to JavaScript promises and can be awaited to get the value once it has finished computing.
- reference
  + a mutable reference to a value. &<type> for the type and &<statement> for a reference value.
- enum
  + wraps any other value with a certain identifier.
  + the type is written as <enum>(<type>): many builtins use Err(String) to report errors.
  + to get a value, use <enum>: <statement>.
