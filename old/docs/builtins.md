# mers builtins

## functions

### assume1

if the input is `[v]`, returns `v`. if the input is `[]`, panics and crashes the program.
useful for prototyping when error-handling is not a priority or for use with the `get` function with an index that is always in range.

### assume_no_enum

if the input is any enum variant, panics and crashes the program.
just like `assume1`, this is useful to quickly ignore `Err(_)` types.

### noenum

if the input is any enum variant, returns the inner value. mostly used in `switch` statements when you want to use the inner value from an enum value (like the `string` from an `Err(string)`).

### matches

returns `[]` for values that don't match and `[v]` for ones that do, where `v` is the matched value.
useful to avoid the (sometimes ambiguous) values that technically match but aren't `[v]`.

### clone

NOTE: may be replaced with dereference syntax?

### print

writes a string to stdout

### println

like print, but adds a newline character

### debug

prints a values compile-time type, runtime type and value

### read_line

reads one line from stdin and returns it.
blocks until input was received.

### to_string

converts any value to a string so that the string, if parsed by the mers parser, would perfectly represent the valu.
Exceptions are strings (because of escape sequences), functions, thread values and maybe more. this list should get shorter with time.

### format

given a format string (first argument) and any additional number of strings, replaces "{n}" in the format string with the {n}th additional argument,
so that {0} represents the first extra argument: `"val: {0}".format(value)`

### parse_int

tries to parse a string to an int

### parse_float

tries to parse a string to a float

### run

runs an anonymous function. further arguments are passed to the anonymous function.

### thread

like run, but spawns a new thread to do the work.

### await

takes the value returned by thread, waits for the thread to finish and then returns whatever the anonymous function used to spawn the thread returned.

### sleep

sleeps for a number of seconds (int/float)

### exit

exits the process, optionally with a specific exit code

### fs_list, fs_read, fs_write

file system operations - will likely be reworked at some point

### bytes_to_string, string_to_bytes

converts UTF-8 bytes to a string (can error) and back (can't error)

### run_command, run_command_get_bytes

runs a command (executable in PATH) with a set of arguments (list of string), returning `[exit_code stdout stderr]` on success

### not

turns `true` to `false` and `false` to `true`

### and, or, add, sub, mul, div, mod, pow, eq, ne, lt, gt, ltoe, gtoe

functions for operators like `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `>`, `<=`, ...

### min, max

returns the max/min of two numbers

### push

given a reference to a list and some value, appends that value to the end of the list.

### insert

same as push, but the index where the value should be inserted can be specified

### pop

given a reference to a list, <removes and returns the last element from a list<

### remove

same as pop, but the index of the value to remove can be specified

### get

given a list and an index, returns the value at that index wrapped in a 1-length tuple or `[]`.
if the first argument is a refernce to a list, this will return a reference to the value at that index (which can be modified):

`&list.get(2).assume1() = "new_value"`

### len

returns the length of the string/list/tuple/...

### contains, starts_with, ends_with

check for certain substring in a string

### index_or

find first index of a certain substring in a string, or return `[]` otherwise

### trim

remove leading and trailing whitespaces from the string

### substring

returns a sustring of the original string.

first argmuent is the start index. -1 is the last character in the string, -2 the second to last and so on.

second argument, if provided, is the end index (exclusive). if it is negative, it limits the string's length: `"1234".substring(1, -2)` returns `"23"`.

### replace

replaces occurences of arg1 in arg0 with arg2

### regex

given a regex (in string form), this function returns either `Err(string)` or a function which, when called with another string, returns a list of matches found in that string:

    lines_regex := regex(".*").assume_no_enum()
    fn lines(s string) {
      lines_regex.run(s)
    }
    debug("a string\nwith multiple\nlines!".lines())

This is done because compiling regex is somewhat slow, so if multiple strings have to be searched by the regex,
it would be inefficient to recompile the regex every time. (btw: credit goes to the Regex crate, which is used to implement this)

### split

given two strings, splits the first one at the pattern specified by the second one:

    word_count := "a string containing five words".split(" ").len()
