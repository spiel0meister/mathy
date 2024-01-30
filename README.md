# mathy

A language meant for easy Math operations!

## Quick Start

Clone the repo, and run:
```console
cargo run -- <file-name>.mth
```

### Syntax

In this language you can:

1. Declare variables:
```mth
x = 2 * 5 + 1
y = x * 2
```

2. Declare functions:
```mth
f(x) = x * 2
g(x, y) = x + 1 / y
```

3. For-like loop--the from-to-as loop:
```mth
from 0 to 10 as x {
    # Stuff...
}
```

You can also use a custom step (defaults to 1.0):
```mth
from 0 to 10 as x with step 2 {
    # Stuff...
}
```

4. Use lists and iterate over them with a for-in loop:
```mth
x = [1, 2, 3]
for y in x {
    y
}
```

5. Print expressions:
```mth
f(x) = x * 2 - 2
from 0 to 10 as x {
    f(x) # The result will be printed out! btw this is a comment, this will be ignored
}
```

## Todo

- [x] Built-in functions (sin, cos, tan, ...)
- [x] Constants (PI, TAU, GOLDEN RATIO, ...)
- [ ] C-like macros to config interpretation (e.g. set angle mode to degrees, radian, or grad, ...)
