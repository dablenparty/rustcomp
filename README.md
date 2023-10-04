# rustcomp

[![crates.io](https://img.shields.io/crates/v/rustcomp)](https://crates.io/crates/rustcomp)
[![docs](https://docs.rs/rustcomp/badge.svg)](https://docs.rs/rustcomp)

Adds idiomatic comprehensions to Rust. This is achieved through a functional macro, `rcomp!`, that expands to iterators. See the [documentation](https://docs.rs/rustcomp) for more information.

Adds comprehensions to Rust. This is achieved through a functional macro, [`rcomp!`], that does all the heavy lifting for you.

## Basic Usage

The core idea is simple: provide an easy and concise way to flatten, map, filter, and collect iterators. A basic comprehension looks like this:

```rust
let v = rcomp![Vec<_>; for x in 0..10 => x];
let it = (0..10).collect::<Vec<_>>(); // all examples show an equivalent iterator
assert_eq!(v, it);
```

This will make a `Vec<i32>` with the numbers 0 through 9... not very useful, is it? Let's add a guard to filter out the odd numbers:

```rust
let v = rcomp![Vec<_>; for x in 0..10 => x, if x % 2 == 0];
let it = (0..10).filter(|x| x % 2 == 0).collect::<Vec<_>>();
assert_eq!(v, it);
```

Now we're getting somewhere! You can also map the values, so let's double the values:

```rust
let v = rcomp![Vec<_>; for x in 0..10 => x * 2, if x % 2 == 0];
let it = (0..10)
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .collect::<Vec<_>>();
assert_eq!(v, it);
```

Notice how the `map` call comes _after_ the `filter` call in the iterator chain. This is also how the comprehension works: the guard applies to the _input_ value, not the output value.

## Note on Collecting

If you don't want to collect the results into a container, you can get the iterator directly by omitting the collection type:

```rust
// now we have to collect the iterator ourselves
let v = rcomp![for x in 0..10 => x].collect::<Vec<_>>();
// equivalent to:
let vv = rcomp![Vec<_>; for x in 0..10 => x];
```

## Destructuring

Comprehensions also support destructuring. For example, tuples:

```rust
let pairs = vec![(1, 2), (3, 4), (5, 6)];
let v = rcomp![Vec<_>; for (x, y) in &pairs => x + y];
let it = pairs.into_iter().map(|(x, y)| x + y).collect::<Vec<_>>();
assert_eq!(v, it);
```

or structs:

```rust
struct Point {
  x: i32,
  y: i32,
}

// impl new, etc...

let points = vec![Point::new(1, 2), Point::new(3, 4), Point::new(5, 6)];
let v = rcomp![Vec<_>; for Point { x, y } in &points => x + y];
let it = points.into_iter().map(|Point { x, y }| x + y).collect::<Vec<_>>();
assert_eq!(v, it);
```

## Nesting

Nested iterators are supported up to the recursion limit by chaining the `in` clauses:

```rust
let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
let v = rcomp![Vec<_>; for row in &matrix, col in row => *col * 2, if *col % 2 == 0];
// nested loops are a much nicer example than iterators here
let mut it = Vec::new();
for row in &matrix {
    for col in row {
        if *col % 2 == 0 {
           it.push(*col * 2);
       }
    }
}
assert_eq!(v, it);
```

## What about `mapcomp`?

I'm aware of the existence of the [`mapcomp`](https://docs.rs/mapcomp/latest/mapcomp/index.html) crate, but it differs from this crate in a few ways. For starters, `mapcomp` aims to make their syntax as close to Python as possible and I think they did a great job; this crate is not trying to do that. The goal of this crate is to add comprehensions to Rust in an idiomatic way with a syntax that flows naturally with the rest of the language while still being concise and powerful. `mapcomp` also provides multiple macros for different types of comprehensions while this crate provides only one.

On a more technical note, `mapcomp` uses generators internally which was okay for Rust 2018, but generators and `yield`-ing are now experimental features. This was a big inspiration for this crate, as I wanted to make a macro-based solution that didn't require nightly, so I settled on iterators in lieu of generators.
