# Hello, world!

Welcome to mers! Start by creating a file for your mers code:

    println("Hello, world!")

now run it: `mers <file>` (replace mers the location of the compiled executable if it's not in your PATH)

# basic concepts

## variables

    a = 15
    println("a: " + a.to_string())
    // clones the value
    b = a
    a = 25
    println("a: " + a.to_string())
    println("b: " + b.to_string())

## if statements

    // type: bool
    condition = true
    if condition {
        println("yes")
    } else {
        println("no")
    }

### else-if statements

    number = 15
    if number == 10 {
        println("ten")
    } else if number == 15 {
        println("fifteen")
    } else {
        println("another number")
    }

## switch statements

    condition = true
    // type: string/int
    val = if condition {
        "some text"
    } else {
        15
    }
    // do different things depending on the type
    switch! val {
        string println("text: " + val)
        int {
            // we need to convert val to a string before we can use it here
            println("number: " + val.to_string())
        }
    }

## match statements

\[...]
