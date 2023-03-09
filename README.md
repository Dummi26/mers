# mers
Mark's error-script -- A script language with explicit error handling and type-checks that run before the script even starts.

**This is highly experimental and a lot of things might change. It lacks very basic builtins (who needs to add integers together anyways) and has a lot of bugs.**

## checks run before the script does

One of the worst things about script languages is writing a script that runs for more than 5 minutes just to crash right before printing the output because of some type-error.
With mers, this won't happen - mers runs checks and parses the script into a special data structure before running it. This ensures that types are always what they should be and all errors can be caught before the script even starts.
(mers was born out of the need for "a script language that handles errors almost as well as Rust, but with less typing")

## Type-Checked

The main feature of this language is the type-checking. Here are some common approaches that you might be familiar with:
- Asssign a type or a Trait/Interface/Base Class/... to each variable (most statically typed languages - Java, C#, and Rust)
- Just don't - what could go wrong? (please stop doing this - JavaScript and most Scripts/Shells)

While the OOP approach is safe and won't cause weird crashes, it requires classes, interfaces or enums, which can be a lot to type for a simple script.

The JavaScript approach requires way less typing, but is prone to crashing

In mers, each variable has a type. However, this type is actually a list of all types the variable can have. This means
- v = "this is a string"
  + s will always be a string. The type-checker knows this.
- v = if some_condition { "Hello, user!" } else { -1 }
  + v could be a string or an integer. The type checker knows this, too, showing its type as [String, Int].
- v = thread( () "This is an anonymous function returning a string" )
  + Here, we use thread to run an anonymous function in the background. v will have the type Thread(String). If we do r = v.await(), r will be a String. The type-checker knows this (TODO! it should, but currently doesn't. prepare for crashes.).

To ensure that variables with more than one possible type won't cause issues, **every possible type has to be handeled**. This can, for example, be achieved using to_string(), which accepts arguments of types String, Int, Float, Tuple, or List.
If a variable could be a String, Int or Float and should be multiplied with another Float, the type-checker will complain that the String case isn't handeled because the mul() function doesn't accept String arguments.
To distinguish between different types, a *switch* statement has to be used. To pattern-match on known types, *match* statements can be used. (TODO! match isn't implemented yet.)

## Error handling

Using Rust has dropped the number of runtime errors/exceptions in my projects significantly. The same thing should be true for mers. Just like Rust, there are no exceptions. If a function can fail, that will be visible in the return type.
In Rust, loading a file might show Result<String, io::Error> as the functions return type. In Mers, it would be String/[], where [] is an empty Tuple similar to () in Rust. The Type-Checker will refuse to assume that the function returned String, because it might have returned nothing.
We need to handle the String case explicitly, for example using *switch*. If we don't, the type-checker wont even let the script start, preventing runtime errors.

## Multithreading

"Function" is a valid type for variables in mers.
Functions have any number of arguments and can capture their environment.
They can be executed synchronously using run(f, args..) or on a different thread using thread(f, args..).
run() will return what the function returns while thread() will return a Thread value. await(thread) will wait for the thread to finish and return its value.

## Examples

### Running a thread and awaiting it, passing arguments to the thread when starting it and sharing a variable because the thread's function captured it (useful for reporting progress, i.e. changing a float from 0.0 to 100.0 while downloading and using the main thread to animate a progress bar, then using .await() only when the float is set to 100 to avoid blocking)

    print( "Starting" )

    captured = "Unchanged"

    calc_function = (prog_message string) {
        sleep( 1 )
        print( prog_message )
        sleep( 3 )
        captured = "changed from a different thread"
        "this is my output"
    }

    calc_thread = thread(calc_function "progress message!")

    sleep( 2 )
    print( "Done." )
    print( captured )

    calc_thread.await().print()
    print( captured )

## Quirks

currently, f(a b c) is the same as a.f(b c). var.function(args) will use var as the function's first argument, moving all other arguments back. This removes the need for struct/class syntax. Simply declare a function scream(str string) { str.to_upper().print() } and you can now use var.scream() on all strings.

function(var) will break the parser, every argument has to be followed by a whitespace (space, tab, newline, ...). This will hopefully be fixed soon, but for now, function(var ) or just var.function() is the syntax that works.
