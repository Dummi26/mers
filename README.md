# mers

Mers is an experimental programming language inspired by high-level and scripting languages, but with error handling inspired by rust.

## WARNING

If you use libraries, be aware that they run as a seperate process that might not exit with mers!
This means that, after running a script 40-50 times (which can happen more quickly than you might realize),
you might find 40-50 random processes just running and possibly maxing out your cpu.
So if you use libraries (recommendation: don't, the implementation is pretty bad anyway. just use any other language), make sure to kill those processes once you're done
until I figure out how to make that happen automatically.
(I believe the issue happens when closing the window from the GUI library, which crashes mers, leaving the http library process running)

(other than that, the language is pretty usable, i promise...)

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

## building mers

to use mers, clone this repo and compile it using cargo. (if you don't have rustc and cargo, get it from https://rustup.rs/. the mers project is in the mers subdirectory, one level deeper than this readme.)

for simplicity, i will assume you have the executable in your path and it is named mers. Since this probably isn't the case, just know that `mers` can be replaced with `cargo run --release` in all of the following commands.

### running from a file

To run a program, just run `mers your_file.txt`. The file needs to be valid utf8.
Alternatively, run `mers -e println("Hello, file-less world")`.
If you compiled mers in debug mode, it will print a lot of debugging information.

### tutor

Use `mers -t` to start the tutor, which will give you an interactive tour of the language.

### interactive mode

Use `mers -i` to start interactive mode. mers will create a temporary file and open it in your default editor. Every time the file is saved, mers reloads and runs it, showing errors or the output.

If your default editor is a CLI editor, it might hide mers' output. Run `mers -i+t` to start the editor in another terminal. This requires that $TERM is a terminal emulator that works with the `TERM -e command args` syntax (alacritty, konsole, ..., but not wezterm).

### Output

Somewhere in mers' output, you will see a line with five '-' characters: ` - - - - -`. This is where your program starts running. The second time you see this line is where your program finished. After this, You will see how long your program took to run and its output.

## Intro

Welcome to mers! This section explains the basics of the language via examples.
Some basic knowledge of another programming language might be helpful,
but no advanced knowledge is required (and if it is, that just means that my examples aren't good enough and need to be improved).

### Hello, World!

Since this is likely your first time using mers, let's write a hello world program:

    println("Hello, World!")

If you're familiar with other programming languages, this is probably what you expected. Running it prints Hello, World! between the two five-dash-lines.
The `"` character starts/ends a string literal. This creates a value of type `String` which is then passed to the `println()` function, which writes the string to the programs standard output (stdout).

### Hello, World?

But what if we didn't print anything?

    "Hello, World?"

Running this should show `Hello, World?` as the program's output. This is because the output of a code block in mers is always the output of its last statement. Since we only have one statement, its output is the entire program's output.

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

`format()` is a builtin function that is used to format text.
It takes a string (called the format string) and any number of further arguments. The pattern `{n}` anywhere in the format string will be replaced with the n-th argument, not counting the format string.

    println("What is your name?")
    name = read_line()
    println(format("Hello, {0}! How was your day?" name))

### alternative syntax for the first argument

If function(a b) is valid, you can also use a.function(b). They do exactly the same thing.
This can make reading `println(format(...))` statements a lot more enjoyable:

    println("Hello, {0}! How was your day?".format(name))

This also works to chain multiple functions together:

    "Hello, {0}! How was your day?".format(name).println()

### A simple counter

Let's build a counter app: We start at 0. If the user types '+', we increment the counter by one. If they type '-', we decrement it. If they type anything else, we print the current count in a status message.

The first thing we will need for this is a loop to prevent the app from stopping after the first user input:

    while {
        println("...")
    }

Running this should spam your terminal with '...'.

Now let's add a counter variable, read user input and print the status message.

    counter = 0
    while {
        input = read_line()
        println("The counter is currently at {0}. Type + or - to change it.".format(counter))
    }

We can then use `eq(a b)` to check if the input is equal to + or -, and then decide to increase or decrease counter:

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

mers actually doesn't have an else-if, the if statement is simply parsed as:

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

Let's replace the if statement from before with a match statement!

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

Match statements are a lot more powerful than if-else-statements, but this will be explained in a later example.

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
    switch! file {}

Since `get()` can fail, it returns `[]/[t]` where t is the type of elements in the list. To avoid handling the `[]` case, the `assume1()` builtin takes a `[]/[t]` and returns `t`. If the value is `[]`, it will cause a crash.

We get the first argument passed to our program and read it from disk using fs_read. To run this: `mers script.txt test_file.txt`.

`switch` is used to distinguish between multiple possible types. `switch!` will cause a compiler error unless *all* types are covered.
This is useful to see what type the compiler thinks a variable has: In this case `[int ...]/Err(string)`.

Err(string) is a string value in an Err enum. Builtin functions use the Err enum to indicate errors because there is no concrete Error type.

After handling all errors, this is the code I came up with:

    file = fs_read(&args.get(0).assume1("please provided a text file to read!"))
    switch! file {
        [int ...] {
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

Some constructs in mers use the concept of "Matching". The most obvious example of this is the `match` statement:

    x = 10
    match x {
        x.eq(10) println("x was 10.")
        true println("x was not 10.")
    }

Here, we check if x is equal to 10. If it is, we print "x was 10.". If not, we print "x was not 10.".
So far, this is almost the same as an if statement.

However, match statements have a superpower: They can change the value of the variable:

    x = 10
    match x {
        x.eq(10) println("x is now {0}".format(x))
        true println("x was not 10.")
    }

Instead of the expected "x is now 10", this actually prints "x is now true", because `x.eq(10)` returned `true`.
In this case, this behavior isn't very useful. However, many builtin functions are designed with matching in mind.

For example, `parse_int()` and `parse_float()` return `[]/int` and `[]/float`. `[]` will not match, but `int` and `float` will.

We can use this to parse a list of strings into numbers: First, try to parse the string to an int. If that doesn't work, try a float. If that also doesn't work, print "not a number".

Using a match statement, this is one way to implement it:

    strings = ["87" "not a number" "25" "14.5" ...]
    for x strings {
        match x {
            x.parse_int() println("int: {0} = 10 * {1} + {2}".format(x x.sub(x.mod(10)).div(10) x.mod(10)))
            x.parse_float() println("float: {0} = {1} + {2}".format(x x.sub(x.mod(1)) x.mod(1)))
            true println("not a number")
        }
    }


Because the condition statement is just a normal expression, we can make it do pretty much anything. This means we can define our own functions and use those in the match statement
to implement almost any functionality we want. All we need to know is that `[]` doesn't match and `[v]` does match with `v`, so if our function returns `["some text"]` and is used in a match statement,
the variable that is being matched on will become `"some text"`.

### Example: sum of all scores of the lines in a string

This is the task:

You are given a string. Each line in that string is either an int, one or more words (a-z only), or an empty line. Calculate the sum of all scores.

- If a line contains an int, its score is just that int: "20" has a score of 20.
- If a line contains words, its score is the product of the length of all words: "hello world" has a score of 5*5=25.
- If a line is empty, its score is 0.

A possible solution in mers looks like this:

    // if there is at least one argument, treat each argument as one line of puzzle input. otherwise, use the default input.
    input = if args.len().gt(0) {
        input = ""
        for arg args {
            input = input.add(arg).add("\n")
        }
        input
    } else {
        "this is the default puzzle input\n312\n\n-50\nsome more words\n21"
        // 4*2*3*7*6*5 = 5040
        // 312
        // 0
        // -50
        // 4*4*5 = 80
        // 21
        // => expected sum: 5403
    }
    // calculates the score for a line of words. returns 1 for empty lines.
    fn word_line_score(line string) {
        line_score = 1
        // \S+ matches any number of non-whitespace characters => split at spaces
        for x line.regex("\\S+").assume_no_enum("word-split regex is valid") {
            line_score = line_score.mul(x.len())
        }
        line_score
    }
    sum = 0
    // .+ matches all lines that aren't empty
    for x input.regex(".+").assume_no_enum("line-split regex is valid") {
        match x {
            x.parse_int() sum = sum.add(x)
            x.word_line_score() sum = sum.add(x)
        }
    }
    sum


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
  + list types are written as a single type enclosed in square brackets with 3 dots before the closing bracket: [string ...].
  + list values are created by putting any number of statements in square brackets, prefixing the closing bracket with ...: ["hello" "mers" "world" ...].
- functions
  + function types are written as `fn((in1_1 in1_2 in1_3 out1) (in2_1 in2_2 out2))`.
  + function values are created using the `(first_arg_name first_arg_type second_arg_name second_arg_type) statement` syntax: `anonymous_power_function = (a int b int) a.pow(b)`.
  + to run anonymous functions, use the run() builtin: `anonymous_power_function.run(4 2)` evaluates to `16`.
  + note: functions are defined using the `fn name(args) statement` syntax and are different from anonymous functions because they aren't values and can be run directly: `fn power_function(a int b int) a.pow(b)` => `power_function(4 2)` => `16`
- thread
  + a special type returned by the thread builtin. It is similar to JavaScript promises and can be awaited to get the value once it has finished computing. Reading the thread example is probably the best way to see how this works.
- reference
  + a mutable reference to a value. `&type` for the type and `&varname` for a reference value.
- enum
  + wraps any other value with a certain identifier.
  + the type is written as `enum(type)`: many builtins use `Err(String)` to report errors.
  + to get a value, use `enum: statement`: `Err: "something went wrong"`
