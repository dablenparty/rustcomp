#![warn(clippy::all, clippy::pedantic)]
//! Adds vector, set, map, and iterator comprehensions to Rust. This
//! is achieved through two types of functional macro: iterator comprehensions
//! and collection comprehensions. Iterator comprehensions return the raw iterator
//! while collection comprehensions return a collection (e.g. `Vec`, `HashSet`, etc.).
//!
//! ## Iterator Comprehensions
//!
//! - [`iter_comp!`] yields values
//! - [`map_iter_comp!`] yields key-value pairs as tuples
//!
//! ## Collection Comprehensions
//!
//! - [`vec_comp!`] returns `Vec`
//! - [`set_comp!`] returns `HashSet`
//! - [`map_comp!`] returns `HashMap`
//!
//! ## How to use
//!
//! For a full explanation of how to use the macros, see the documentation for
//! `vec_comp!`. `map_comp!` is a bit different, but it has examples as well.
//!
//! ## What about `mapcomp`?
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

// TODO: restructure docs to be easier to navigate

/// Performs a special comprehension that returns a `HashMap`. This is
/// different from the other comprehensions in that it requires two expressions:
/// one for the key and one for the value. For more exhaustive documentation
/// and usage examples, see [`vec_comp!`].
///
/// # Example
///
/// Mapping last names to first names:
///
/// ```rust
/// # use rustcomp::map_comp;
/// let names = vec![("John", "Smith"), ("Jane", "Doe"), ("Bob", "Smith")];
/// let m = map_comp! {for (first, last) in names => last, first, if last == "Smith"};
/// # assert_eq!(m, vec![("Smith", "John"), ("Smith", "Bob")].into_iter().collect::<std::collections::HashMap<_, _>>());
/// ```
#[macro_export]
macro_rules! map_comp {
    ($($t:tt)*) => (
        $crate::map_iter_comp!($($t)*)
            .collect::<::std::collections::HashMap<_, _>>()
    );
}

/// Generates an iterator that yields the results of the comprehension. This
/// macro is specialized for `HashMap`s. For more information, see
/// [`map_comp!`].
#[macro_export]
macro_rules! map_iter_comp {
    // the implementation is EXTREMELY similar to iter_comp, but not identical
    (@__ for $($vars:pat),+ in $iter:expr; $($recurse:tt)+) => {
        $iter
            .into_iter()
            .flat_map(|$($vars),*| $crate::map_iter_comp!(@__ $($recurse)+))
    };
    (@__ for $($vars:pat),+ in $iter:expr => $keymap:expr, $valmap:expr $(, if $guard:expr)? $(,)?) => {
        $iter
            .into_iter()
            .filter_map(|$($vars),*| {
                // `&& true` is a trick to make the guard optional
                if $($guard &&)? true {
                    Some(($keymap, $valmap))
                } else {
                    None
                }
            })
    };
    (for $($t:tt)+) => {
        $crate::map_iter_comp!(@__ for $($t)+)
    };
}

/// Performs a comprehension and returns a `Vec` containing the results. If you
/// want the raw iterator, you can use the [`iter_comp!`] macro (that's what
/// this macro uses internally anyway).
///
/// # Usage
///
/// The core idea is simple: provide an easy and concise way to map, filter,
/// and flatten iterators. For example, a basic comprehension looks like this:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x];
/// # assert_eq!(v, (0..10).collect::<Vec<_>>());
/// ```
///
/// This will make a vector with the numbers 0 through 9... not very useful,
/// is it? Let's add a filter:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x, if x % 2 == 0];
/// # assert_eq!(v, (0..10).filter(|x| x % 2 == 0).collect::<Vec<_>>());
/// ```
///
/// Now we only get even numbers. We can also map the values:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x * 2, if x % 2 == 0];
/// # assert_eq!(v, (0..10).filter(|x| x % 2 == 0).map(|x| x * 2).collect::<Vec<_>>());
/// ```
///
/// Now we get the even numbers, doubled. What's better than that? Well,
/// thanks to Rust's expression syntax, you can destructure tuples:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// let pairs = vec![(1, 2), (3, 4), (5, 6)];
/// let v = vec_comp![for (x, y) in pairs => x + y];
/// # assert_eq!(v, vec![3, 7, 11]);
/// ```
///
/// and structs:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// # struct Point {
/// #    x: i32,
/// #   y: i32,
/// # }
/// let points = //...
/// # vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }, Point { x: 5, y: 6 }];
/// let v = vec_comp![for Point { x, y } in points => x + y];
/// # assert_eq!(v, vec![3, 7, 11]);
/// ```
///
/// and anything else that destructures! You can also nest comprehensions:
///
/// ```rust
/// # use rustcomp::vec_comp;
/// let matrix = //...
/// # vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
/// let v = vec_comp![for row in matrix; for col in row => col, if col % 2 == 0];
/// # assert_eq!(v, vec![2, 4, 6, 8]);
/// ```
///
/// We could keep going with the nesting, but I think you get the idea.
#[macro_export]
macro_rules! vec_comp {
    [$($t:tt)*] => {
        $crate::iter_comp!($($t)*).collect::<Vec<_>>()
    };
}

/// Performs a comprehension and returns a `HashSet` containing the results. For
/// more information, see [`vec_comp!`].
///
/// # Example
/// ```rust
/// # use rustcomp::set_comp;
/// let matrix = //...
/// # vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
/// let v = set_comp![for row in matrix; for col in row => col, if col % 2 == 0];
/// # assert_eq!(v, vec![2, 4, 6, 8].into_iter().collect::<std::collections::HashSet<_>>());
/// ```
#[macro_export]
macro_rules! set_comp {
    ($($t:tt)*) => {
        $crate::iter_comp!($($t)*).collect::<::std::collections::HashSet<_>>()
    };
}

/// Generates an iterator that yields the results of the comprehension.
/// For more information, see [`vec_comp!`].
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
    fn test_nested_map_comp() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let expected = v
            .clone()
            .into_iter()
            .flatten()
            .filter(|x| x % 2 == 0)
            .map(|x| (x.to_string(), x))
            .collect::<std::collections::HashMap<_, _>>();
        let actual = map_comp! {for row in v; for x in row => x.to_string(), x, if x % 2 == 0};
        assert_eq!(expected, actual);
    }

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
