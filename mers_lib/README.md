# mers-lib

The library behind [mers](https://github.com/Dummi26/mers).

With this, you can parse, compile, check and run mers code.
You can also add your own functions and types which can then be used from mers, if you really want to.

## Running mers

There are four steps to running mers code.
The examples show you how to actually implement them,
this readme only explains what they do any why.

### 1. Parsing

This first step converts the source code, a string, to a parsed mers statement.

In this step, syntax errors and unknown variables are caught.

### 2. Compiling

This converts a parsed mers statement to a compiled one. It almost never produces an error.

### 3. Checking

This step is optional. If you parse and compile your source code, you can (try to) run it.
However, mers assumes that all mers code you run is actually valid,
so if you don't check your codes validity, mers will probably panic while running your code.

This step performs all the type-checking and determines the output type of your code, if it is valid.

For example, the following code is valid and has the return type `Int/Float`:

```
my_condition := true

if my_condition {
  5
} else {
  1.4
}
```

### 4. Running

This step assumes that the code it is running is actually valid, so it never returns an error.
As long as `check` didn't return an error in Step 3, it is safe to assume that this will return the value produced by the code.
We can also assume that the return value has a type which is included in that determined by `check`.
If `check` returned an error, this will likely panic.
