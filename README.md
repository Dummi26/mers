# mers

See [the mers readme](mers/README.md) for more info.

---

```
"Hello, World!".println
```

> `Hello, World!`

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

![err1](https://github.com/Dummi26/mers/assets/67615357/2f113287-1cce-427f-8dcb-577841e40c2c)

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
