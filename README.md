```ax
struct Foo {
  some_field: usize
}

impl Foo {
  fn some_method(&self)(a: usize, b: usize) -> usize {
    self.some_field * a + b
  }

  fn some_property(&self) {
    return 4;
  }
}

let x: Foo = Foo { some_field: 3 };

// Unbound function access (similar to Foo::some_method in other languages)
x.some_method

// Bound function access (`self` is now partially applied, passing `x:some_method` into a function allows it to access `self`)
x:some_method

// Can also access properties, like so:
x:copy()

```



```l
let x = 5;
mut x = 5;

loop {
  x = x + 1;
}

let x = {
  do_some_stuff();
  3
};
````


# Keywords

- `loop`
- `if`
- `else`
- `fn`
- `let`
- `mut`
- `break`
- `continue`
- `return`


```l
fn foo(arg1, arg2) {
  mut a = arg1 + arg2;

  mut i = 0;
  loop {
    if (i >= 5) {
      break;
    }
    a = a + 1;
    i = i + 1;
  }

  return a;
}

let x = 3;

let y = foo(x, 7);

print("x =", x, "(15)");
```
