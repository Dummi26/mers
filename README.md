# mers ![build status](https://github.com/Dummi26/mers/actions/workflows/rust.yml/badge.svg)

Mers is a simple and reliable programming language.

While mers can already be used right now,
many things about it will still change -
at least until 1.0.0 is released.
For serious projects, this probably
isn't the language you should be using.

## Why mers?

### it doesn't crash

If your program starts, **it won't crash**.
The only exceptions to this are
the builtin `exit()` function, which can exit with a nonzero exit code
and the builtin `assume` functions, similar to Rust's `unwrap()`.

This is because **errors in mers are values**,
and the compiler forces you to handle them.

### type system

In mers, a type can consist of multiple types:

```
// bool
half := true
// float/int
number := if half { 0.5 } else { 1 }

// some math functions that work with int and float
fn absolute_value(num int/float) {
  if num >= 0 { num } else { 0 - num }
}
fn difference(a int/float, b int/float) {
  absolute_value(a - b)
}

debug(difference(0.2, 2)) // float: 1.8
// although difference can work with floats, since we only give it ints, it also returns an int.
// this is known at compile time.
debug(difference(20, 5)) // int: 15
```

With this, you can write code like you would in dynamically typed languages:

```
user_input := if true { "some text input" } else { 19 }
```

But you get the compile-time checks you love from statically typed ones:

```
if user_input >= min_age {
  println("welcome!")
}
```

Because `user_input` could be a string, the code above won't compile.

### it's simple

Almost everything in mers is built around this type system.
This eliminates many concepts you may be familiar with:

#### instead of null or exceptions, just add `[]` or the error(s) to the return type

```
// returns `[]` instead of dividing by zero
fn div_checked(a int, b int) -> int/[] {
  if b != 0 {
    a / b
  } else {
    []
  }
}

// returns the file content as a string.
// if the file can't be read, returns IoError and an error message.
// if the file's contents weren't Utf8, return NotUtf8 and the file's bytes.
fn load_file_as_string(path string) -> string/IoError(string)/NotUtf8([int ...]) {}
```

#### structs are nothing more than tuples

```
// define our type named Person
type Person [int, string]

// function to get (but not set) the person's birth year
fn birth_year(self Person) self.0

// function to get or set the person's name
fn name(self Person/&Person) self.1

person := [1999, "Ryan"]

// Get the values
person.name() // Ryan
person.birth_year() // 1999

// Change the name
&person.name() = "James"
// or: name(&person) = "James"
person.name() // James

// This doesn't compile:
// &person.birth_year() = 1998
```

#### enums are already part of the type system

```
type DirectoryEntry File(string)/Dir(string)/Link(DirectoryEntry)
```

#### return and break

Most people agree that `goto` statements are usually a bad idea.
But `return` and `break` have similar issues, although less severe:

**they create weird control flow:**

Maybe you want to see how long a function takes to execute, so
you add a `println` to the end - just to see absolutely no output.
Now you might think the function is stuck in an infinite loop somewhere,
although it just returned before it ever got to your println.
This can be annoying. If we remove `return` statements,
it becomes way easier to tell when code will or won't be executed.

**return is exclusive to functions, break to loops**

With mers, whenever possible, concepts in the language should work on *statements*.

An example of this are type annotations:

```
fn half(num int) -> int { num / 2 }
```

The `-> int` indicates that this function returns an int.
Actually, it indicates that the statement following it returns an int.
This obviously means the function will also return an int,
but since this syntax isn't specifically made for functions,
we can use it anywhere we want:

```
num := -> int { 4 + 5 }
half(-> int { double * 2})
```

**So how do we return values then?**
Simple!

```
// a function is just another statement
fn return_int() 4 + 5

// a block returns whatever its last statement returns,
fn return_string() {
  a := "start"
  b := "end"
  a + " " + b
}

// this returns string/[], because there is no else
if condition() {
  "some value"
}

// this returns string/int
if condition() {
  "some value"
} else {
  12
}
```

Most returns should be relatively intuitive,
although some special ones (like loops)
use matching (see docs/intro)

### it has references

To explain why this is necessary,
let's look at examples from other languages.

Here is some JavaScript:

```js
function doSmth(list) {
  list[0] = 0
}
function modify(num) {
  num = 0
}
list = [1]
num = 1
doSmth(list)
modify(num)
console.log(list) // [ 0 ]
console.log(num) // 1
```

The same thing in Go:

```go
package main
import "fmt"

func doSmth(list []int) {
    list[0] = 0
}
func modify(num int) {
    num = 0
}
func main() {
    list := []int{1}
    num := 1
    doSmth(list)
    modify(num)
    fmt.Println(list) // [0]
    fmt.Println(num) // 1
}
```

In both cases, the function was able to modify the list's contents,
but unable to change the number to a different value.
This is not just inconsistent, it's also not always what you wanted to do:

- i passed the list by value, how could the function change the inner value?
- the function is called `modify`, why can't it modify the value?
  + In Go, we could use references to make this work - but that just makes the list example even more annoying, since we don't need a reference to change the inner value.

So, let's look at mers:

```
fn doSmth(list [int ...]) {
  &list.get(0).assume1() = 0
}
fn modify(num &int) {
  num = 0
}
list := [1 ...]
num := 1
doSmth(list)
modify(&num)
debug(list) // [1 ...]
debug(num) // 0
```

The list is unchanged, but the number was changed by `modify`,
because it was explicitly passed as a reference `&int`.
**Unless you pass a reference, the value you passed will not be changed.**

### compile-time execution via macros

```
// at compile-time, runs primes.mers to find the first few prime numbers
primes := !(mers "primes.mers")

// at compile-time, runs the code to create a string
// containing 256 '-'s
long_string := !(mers {
  str := "-"
  loop {
    str = str + str
    if str.len() >= 256 {
      str
    }
  }
})
```

## How mers?

There are prebuilt binaries in `build_scripts/`.

Mers is written in Rust. If you have `cargo`, you can also use the build script in `build_scripts/` to build it yourself.

Now, create a new text file (or choose one from the examples) and run it: `mers <file>`.

## Known Issues (only major ones)

### Multithreading

If a function is called from two threads, all local variables of that function are shared.
This doesn't affect builtin functions, and since functions usually don't take long to execute,
the chance of anyone encountering this is low, but it's something to be aware of.
It's a simple fix in theory, but a lot of work to implement, which is why the bug isn't fixed yet.

## Docs

[intro](docs/intro.md)

[syntax cheat sheet](docs/syntax_cheat_sheet.md)

[builtins](docs/builtins.md)

[statements](docs/statements.md)
