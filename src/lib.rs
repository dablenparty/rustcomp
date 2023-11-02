#![warn(clippy::all, clippy::pedantic)]
/*!
Adds comprehensions to Rust. This is achieved through a functional macro,
[`rcomp!`], that does all the heavy lifting for you.

# Basic Usage

The core idea is simple: provide an easy and concise way to flatten, filter,
map, and collect iterators. For a full breakdown of the syntax, see
the docs for the [`rcomp!`] macro. For now, consider this simple example:

```rust
# use rustcomp::rcomp;
let v = rcomp![Vec<_>; for x in 0..10 => x];
let it = (0..10).collect::<Vec<_>>(); // all examples show an equivalent iterator
assert_eq!(v, it);
```

This will make a `Vec<i32>` with the numbers 0 through 9... not very useful,
is it? Let's add a guard to filter out the odd numbers:

```rust
# use rustcomp::rcomp;
let v = rcomp![Vec<_>; for x in 0..10 => x, if x % 2 == 0];
let it = (0..10).filter(|x| x % 2 == 0).collect::<Vec<_>>();
assert_eq!(v, it);
```

Now we're getting somewhere! You can also map the values, so let's double
them for fun:

```rust
# use rustcomp::rcomp;
let v = rcomp![Vec<_>; for x in 0..10 => x * 2, if x % 2 == 0];
let it = (0..10)
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .collect::<Vec<_>>();
assert_eq!(v, it);
```

Notice how the `map` call comes _after_ the `filter` call in the iterator example.
This is also how the comprehension works: the guard applies to the _input_ value,
not the output value.

Speaking of iterators, if you don't want to collect the results into a container,
you can get the iterator directly by omitting the collection type:

```rust
# use rustcomp::rcomp;
// now we have to collect the iterator ourselves
let v = rcomp![for x in 0..10 => x].collect::<Vec<_>>();
// equivalent to:
let vv = rcomp![Vec<_>; for x in 0..10 => x];
# let it = (0..10)
#     .collect::<Vec<_>>();
# assert_eq!(v, vv);
# assert_eq!(v, it);
```

# Destructuring

Comprehensions also support destructuring. For example, tuples:

```rust
# use rustcomp::rcomp;
let pairs = vec![(1, 2), (3, 4), (5, 6)];
let v = rcomp![Vec<_>; for (x, y) in &pairs => x + y];
let it = pairs.into_iter().map(|(x, y)| x + y).collect::<Vec<_>>();
assert_eq!(v, it);
```

or structs:

```rust
# use rustcomp::rcomp;
struct Point {
  x: i32,
  y: i32,
}
#
# impl Point {
#    fn new(x: i32, y: i32) -> Self {
#       Self { x, y }
#   }
# }

let points = vec![Point::new(1, 2), Point::new(3, 4), Point::new(5, 6)];
let v = rcomp![Vec<_>; for Point { x, y } in &points => x + y];
let it = points.into_iter().map(|Point { x, y }| x + y).collect::<Vec<_>>();
assert_eq!(v, it);
```

# Flattening

Flattening nested iterators is supported up to the recursion
limit by chaining the `for-in` clauses:

```rust
# use rustcomp::rcomp;
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

# Advanced Examples

See the [`rcomp!`] macro documentation for some advanced examples,
like creating a `HashMap` or `HashSet`.

# Note on Iterator Examples

It's important to note that iterator examples used to test the
comprehensions are _equivalent_ to the comprehensions, but not
_identical_. The macro expands to nested chains of `flat_map`
and `filter_map` calls; the examples are written for clarity
and to show the order of operations in the comprehension. For
example, the matrix example from earlier expands to:

```rust
# use rustcomp::rcomp;
# let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
let v = (&matrix)
    .into_iter()
    .flat_map(|row| {
        row.into_iter().filter_map(|col| {
            if (*col % 2 == 0) && true {
                Some((*col * 2))
            } else {
                None
            }
        })
    })
    .collect::<Vec<_>>();
# let mut it = Vec::new();
# for row in &matrix {
#     for col in row {
#         if *col % 2 == 0 {
#            it.push(*col * 2);
#        }
#     }
# }
# assert_eq!(v, it);
```

Notice the use of `into_iter` in the expansion.

# What about `mapcomp`?

I'm aware of the existence of the [`mapcomp`](https://docs.rs/mapcomp/latest/mapcomp/index.html)
crate, but it differs from this crate in a few ways. For starters,
`mapcomp` aims to make their syntax as close to Python as possible and
I think they did a great job; this crate is not trying to do that. The
goal of this crate is to add comprehensions to Rust in an idiomatic way
with a syntax that flows naturally with the rest of the language while
still being concise and powerful. `mapcomp` also provides multiple
macros for different types of comprehensions while this crate provides
only one.

On a more technical note, `mapcomp` uses generators internally which was
okay for Rust 2018, but generators and `yield`-ing are now experimental
features. This was a big inspiration for this crate, as I wanted to make
a macro-based solution that didn't require nightly, so I settled on iterators
in lieu of generators.
*/

/// Generates an iterator that yields the results of the comprehension. The
/// syntax allows for flattening, filtering, mapping, and collecting iterators
/// (in that order).
///
/// There are 4 main components to a comprehension:
/// - The optional collection type, which is passed to a `collect` call by the
///   macro. If this is omitted, the macro will return an iterator instead of
///   a collection.
/// - The `for-in` clause, which iterates over the input(s). This can be
///   chained (e.g. `for i in v1, j in v2, k in v3, ...`) to flatten nested
///   iterators, up to the recursion limit.
/// - The mapping expression, which transforms the input.
/// - The optional guard expression, which filters the input. Although this is
///   the last component of the comprehension, it is applied _before_ the
///   mapping expression. In a sense, the end of the comprehension looks like
///   a `match` arm. This has a few implications which are explored more in
///   the [crate-level documentation](crate).
///
/// With that explained, here's the full syntax:
///
/// ```text
/// rcomp!([collect_ty;] for <pattern> in <iterator>, ... => <mapper>[, if <guard>]);
/// ```
///
/// # Examples
///
/// Comprehensions can be as simple or complex as you want. They can collect
/// the input, filter it, map it, and flatten it all in one go. For example,
/// here's how you can create a `HashMap` of numbers and their squares using
/// a comprehension:
///
/// ```rust
/// # use rustcomp::rcomp;
/// # use std::collections::HashMap;
/// let m = rcomp![HashMap<_, _>; for i in 0..10 => (i, i * i)];
/// let it = (0..10).map(|i| (i, i * i)).collect::<HashMap<_, _>>();
/// assert_eq!(m, it);
/// ```
///
/// Another example is removing duplicates from a `Vec` by converting it to
/// a `HashSet` and back:
///
/// ```rust
/// # use rustcomp::rcomp;
/// # use std::collections::HashSet;
/// let v = vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
/// let s = rcomp![Vec<_>; for i in rcomp![HashSet<_>; for j in &v => *j] => i];
/// assert!(v.into_iter().all(|i| s.contains(&i)));
/// ```
///
/// See the [crate-level documentation](crate) for more examples.
#[macro_export]
macro_rules! rcomp {
    (@__ $($vars:pat),+ in $iter:expr => $mapper:expr $(, if $guard:expr)? $(,)?) => (
        $iter
            .into_iter()
            .filter_map(|$($vars),*| {
                // `&& true` is a trick to make the guard optional
                if $($guard &&)? true {
                    Some($mapper)
                } else {
                    None
                }
            })
    );
    (@__ $($vars:pat),+ in $iter:expr, $($recurse:tt)+) => (
        $iter
            .into_iter()
            .flat_map(|$($vars),*| $crate::rcomp!(@__ $($recurse)+))
    );
    // these two rules MUST stay in this order, otherwise the `for`
    // keyword causes ambiguity. the tt munching shouldn't go too
    // deep since it has an end condition.
    (for $($t:tt)*) => (
        $crate::rcomp!(@__ $($t)*)
    );
    ($collect:path; $($t:tt)*) => (
        $crate::rcomp!($($t)*)
        .collect::<$collect>()
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_vec_comp() {
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v
            .clone()
            .into_iter()
            .filter_map(|i| if i % 2 == 0 { Some(i.into()) } else { None })
            .collect::<Vec<u64>>();
        // collect evens
        let actual = rcomp![Vec<_>; for x in v => u64::from(x), if x % 2 == 0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_vec_comp() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let expected = v
            .clone()
            .into_iter()
            .flatten()
            .filter(|x| x % 2 == 0)
            .collect::<Vec<_>>();
        let actual = rcomp![Vec<_>; for row in v, x in row => x, if x % 2 == 0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_full_comprehension() {
        // essentially a no-op
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v
            .clone()
            .into_iter()
            .filter_map(|i| if i % 2 == 0 { Some(i.into()) } else { None })
            .collect::<Vec<u64>>();
        // collect evens
        let actual = rcomp![Vec<_>; for x in v => u64::from(x), if x % 2 == 0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_useless_comp() {
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v.clone().into_iter().collect::<Vec<u32>>();
        let actual = rcomp![for x in v => x].collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_multiple_vars() {
        let v: Vec<(i32, i32)> = vec![(1, 2), (3, 4), (5, 6)];
        let expected: Vec<i32> = v.clone().into_iter().map(|(_, y)| y).collect();
        let actual = rcomp![for (_, y) in v => y].collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }
}
