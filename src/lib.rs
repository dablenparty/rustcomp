#![warn(clippy::all, clippy::pedantic)]

macro_rules! iter_comp {
    ($mapper:expr => for $($vars:pat),* in $iter:expr $(; if $guard:expr)? $(,)?) => {
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
    fn test_full_comprehension() {
        // essentially a no-op
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v
            .clone()
            .into_iter()
            .filter_map(|i| if i % 2 == 0 { Some(i.into()) } else { None })
            .collect::<Vec<u64>>();
        // collect evens
        let actual = iter_comp!(u64::from(x) => for x in v; if x % 2 == 0).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_useless_comp() {
        let v: Vec<u32> = vec![1, 2, 3, 4, 5];
        let expected = v.clone().into_iter().collect::<Vec<u32>>();
        let actual = iter_comp!(x => for x in v).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_multiple_vars() {
        let v: Vec<(i32, i32)> = vec![(1, 2), (3, 4), (5, 6)];
        let expected: Vec<i32> = v.clone().into_iter().map(|(_, y)| y).collect();
        let actual = iter_comp!(y => for (_, y) in v).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }
}
