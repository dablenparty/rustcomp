#![warn(clippy::all, clippy::pedantic)]
//! Adds vector, set, map, and iterator comprehensions to Rust. This
//! is achieved through a functional macro, [`iter_comp!`], that
//! expands to iterators.
//!
//! # Usage
//!
//! ## Basic Usage
//!
//! The core idea is simple: provide an easy and concise way to map, filter,
//! and flatten iterators. These examples use [`vec_comp!`] to keep things
//! short and neat, but all comprehension macros use the same syntax.
//! With that said, a basic comprehension looks like this:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! let v = vec_comp![for x in 0..10 => x];
//! let it = (0..10).collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! This will make a vector with the numbers 0 through 9... not very useful,
//! is it? Let's only keep the evens by adding a guard expression:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! let v = vec_comp![for x in 0..10 => x, if x % 2 == 0];
//! let it = (0..10).filter(|x| x % 2 == 0).collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! Now we're getting somewhere! You can also map the values. For example,
//! let's double the values:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! let v = vec_comp![for x in 0..10 => x * 2, if x % 2 == 0];
//! let it = (0..10)
//!     .filter(|x| x % 2 == 0)
//!     .map(|x| x * 2)
//!     .collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! Notice how the `map` call comes _after_ the `filter` call in the iterator chain.
//! This is to show how the comprehension works: the guard applies to the _input_ value,
//! not the output value.
//!
//! ## Destructuring
//! Comprehensions also support destructuring, for example, tuples:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! let pairs = vec![(1, 2), (3, 4), (5, 6)];
//! let v = vec_comp![for (x, y) in &pairs => x + y];
//! let it = pairs.into_iter().map(|(x, y)| x + y).collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! or structs:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! struct Point {
//!   x: i32,
//!   y: i32,
//! }
//!
//! let points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }, Point { x: 5, y: 6 }];
//! let v = vec_comp![for Point { x, y } in &points => x + y];
//! let it = points.into_iter().map(|Point { x, y }| x + y).collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! ## Nesting
//! Nested iterators are supported up to the recursion limit:
//!
//! ```rust
//! # use rustcomp::vec_comp;
//! let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
//! // also a good example of passing a reference to the comprehension
//! let v = vec_comp![for row in &matrix; for col in row => *col * 2, if *col % 2 == 0];
//! let it = matrix
//!     .into_iter()
//!     .flatten()
//!     .filter(|x| x % 2 == 0)
//!     .map(|x| x * 2)
//!     .collect::<Vec<_>>();
//! assert_eq!(v, it);
//! ```
//!
//! ## Advanced Examples
//!
//! See the [`iter_comp!`] macro documentation for some advanced examples,
//! like creating a `HashMap` from a comprehension.
//!
//! ## Note on Examples
//!
//! It's important to note that iterator examples used to test the
//! comprehensions are _equivalent_ to the comprehensions, but not
//! _identical_. The macros expand to nested chains of `flat_map`
//! and `filter_map` calls; the examples are written for clarity
//! and to show the order of operations in the comprehension.
//!
//! # What about `mapcomp`?
//!
//! I'm aware of the existence of the [`mapcomp`](https://docs.rs/mapcomp/latest/mapcomp/index.html)
//! crate, although it differs from this crate in a few ways. For starters,
//! `mapcomp` aims to make their syntax as close to Python as possible and
//! I think they did a great job; this crate is not trying to do that. The
//! goal of this crate is to add comprehensions to Rust in an idiomatic way
//! so that the syntax flows naturally with the rest of the language while
//! still being concise and powerful.
//!
//! On a more technical note, `mapcomp` uses generators internally which was
//! okay for Rust 2018, but generators and `yield`-ing are now experimental
//! features. This was a big inspiration for this crate, as I wanted to make
//! a macro-based solution that didn't require nightly, so I settled on iterators
//! in lieu of generators.

/// Convenience macro that wraps [`iter_comp!`] and returns a `Vec` containing
/// the results.
///
/// # Example
///
/// ```rust
/// # use rustcomp::{iter_comp, vec_comp};
/// let v = vec_comp![for x in 0..10 => x, if x % 2 == 0];
/// // is equivalent to:
/// let it = iter_comp!(for x in 0..10 => x, if x % 2 == 0).collect::<Vec<_>>();
/// assert_eq!(it, v);
/// ```
#[macro_export]
macro_rules! vec_comp {
    [$($t:tt)*] => {
        $crate::iter_comp!($($t)*).collect::<Vec<_>>()
    };
}

/// Convenience macro that wraps [`iter_comp!`] and returns a `HashSet` containing
/// the results.
///
/// # Example
///
/// ```rust
/// # use rustcomp::{iter_comp, set_comp};
/// # use std::collections::HashSet;
/// let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
/// let s = set_comp![for row in &matrix; for col in row => *col, if col % 2 == 0];
/// // is equivalent to:
/// let it = iter_comp!(for row in matrix; for col in row => col, if col % 2 == 0).collect::<HashSet<_>>();
/// assert_eq!(s, it);
/// ```
#[macro_export]
macro_rules! set_comp {
    ($($t:tt)*) => {
        $crate::iter_comp!($($t)*).collect::<::std::collections::HashSet<_>>()
    };
}

/// Generates an iterator that yields the results of the comprehension. The
/// syntax allows for filtering, mapping, and flattening iterators (in that
/// order).
///
/// There are 3 main components to a comprehension:
/// - The `for-in` clause, which iterates over the input
/// - The guard expression, which filters the input
/// - The mapping expression, which transforms the input. If the guard is
///   present, the mapping expression is only applied to values that pass
///   the guard.
///
/// # Examples
///
/// Comprehensions can be as simple or complex as you want. They can collect
/// the input, filter it, map it, and flatten it all in one go. For example,
/// here's how you can create a `HashMap` of numbers and their squares using
/// a comprehension:
///
/// ```rust
/// # use rustcomp::iter_comp;
/// # use std::collections::HashMap;
/// let m = iter_comp!(for i in 0..10 => (i, i * i)).collect::<HashMap<_, _>>();
/// let it = (0..10).map(|i| (i, i * i)).collect::<HashMap<_, _>>();
/// assert_eq!(m, it);
/// ```
///
/// See the [crate-level documentation](crate) for more examples.
#[macro_export]
macro_rules! iter_comp {
    (@__ for $($vars:pat),+ in $iter:expr; $($recurse:tt)+) => (
        $iter
            .into_iter()
            .flat_map(|$($vars),*| $crate::iter_comp!(@__ $($recurse)+))
    );
    (@__ for $($vars:pat),+ in $iter:expr => $mapper:expr $(, if $guard:expr)? $(,)?) => (
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
    (for $($t:tt)+) => (
        $crate::iter_comp!(@__ for $($t)+)
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
        let actual = vec_comp![for x in v => u64::from(x), if x % 2 == 0];
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
        let actual = vec_comp![for row in v; for x in row => x, if x % 2 == 0];
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
        let actual = iter_comp!(for x in v => u64::from(x), if x % 2 == 0).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_useless_comp() {
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v.clone().into_iter().collect::<Vec<u32>>();
        let actual = iter_comp!(for x in v => x).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_multiple_vars() {
        let v: Vec<(i32, i32)> = vec![(1, 2), (3, 4), (5, 6)];
        let expected: Vec<i32> = v.clone().into_iter().map(|(_, y)| y).collect();
        let actual = iter_comp!(for (_, y) in v => y).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }
}
