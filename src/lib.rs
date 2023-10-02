#![warn(clippy::all, clippy::pedantic)]

/// This macro was designed to add Python-like list comprehensions to Rust.
/// It returns an vector containing the results of the comprehension. If
/// you want the raw iterator, you can use the [`iter_comp!`] macro (that's
/// what this macro uses internally).
///
/// # How to use
/// The core idea is the same as Python, although the syntax is a bit different.
/// A simple comprehension looks like this:
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x];
/// # assert_eq!(v, (0..10).collect::<Vec<_>>());
/// ```
/// This will make a vector with the numbers 0 through 9... not very useful,
/// is it? Let's add a filter:
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x, if x % 2 == 0];
/// # assert_eq!(v, (0..10).filter(|x| x % 2 == 0).collect::<Vec<_>>());
/// ```
/// Now we only get even numbers. We can also map the values:
/// ```rust
/// # use rustcomp::vec_comp;
/// let v = vec_comp![for x in 0..10 => x * 2, if x % 2 == 0];
/// # assert_eq!(v, (0..10).filter(|x| x % 2 == 0).map(|x| x * 2).collect::<Vec<_>>());
/// ```
/// Now we get the even numbers, doubled. What's better than that? Well,
/// thanks to Rust's expression syntax, you can destructure tuples:
/// ```rust
/// # use rustcomp::vec_comp;
/// let pairs = vec![(1, 2), (3, 4), (5, 6)];
/// let v = vec_comp![for (x, y) in pairs => x + y];
/// # assert_eq!(v, vec![3, 7, 11]);
/// ```
/// and structs:
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
/// and anything else that destructures!
#[macro_export]
macro_rules! vec_comp {
    [$($t:tt)*] => {
        $crate::iter_comp!($($t)*).collect::<Vec<_>>()
    };
}

/// Generates an iterator that yields the results of the comprehension.
/// For more information, see [`vec_comp!`].
#[macro_export]
macro_rules! iter_comp {
    (for $($vars:pat),* in $iter:expr => $mapper:expr $(, if $guard:expr)? $(,)?) => {
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
    };
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
            .filter_map(|v| {
                if v.iter().any(|&i| i % 2 == 0) {
                    Some(v)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let actual = vec_comp![for row in v; for x in row => row, if x % 2 == 0];
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
