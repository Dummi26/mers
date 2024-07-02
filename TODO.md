# IN CODE

rm mers_lib/src/program/configs/util.rs

# Objects

```
o := {
  c: 12
  x: 5.0
  y: 3.2
  coords: o -> o.c
  println: o -> "aaa".println
}

{ println: println } := o
o.println
```
