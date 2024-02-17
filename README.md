# mers

See [mers/README.md]

---

```
"Hello, World!".println
```

> `Hello, Variable!`

---

```
my_var := "Hello, Variable!"
my_var.println
```

> `Hello, Variable!`

---

```
(1, 2, 3, 4).sum.println
```

> `10`

---

```
(1, "2", 3, 4).sum.println
```

---

```
(1, 2, 3, 4).as_list.debug
```

> `List<Int> :: [1, 2, 3, 4]`

---

```
(1.0, 2.0).as_list.debug
```

> `List<Float> :: [1, 2]`

---

```
(1, 2, 3.5).as_list.debug
```

> `List<Int/Float> :: [1, 2, 3.5]`

---

```
int_list := (1, 2, 3).as_list
float_list := (4.5, 6.0).as_list
int_list.chain(float_list).as_list.debug
```

> `List<Int/Float> :: [1, 2, 3, 4.5, 6]`
