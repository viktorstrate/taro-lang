use std::cmp::min;

pub fn lev_dist(src: &str, tar: &str) -> usize {
    let n = tar.len() + 1;

    let mut v0 = vec![0usize; n];
    let mut v1 = vec![0usize; n];

    for i in 1..n {
        v0[i] = i;
    }

    for (i, s) in src.chars().enumerate() {
        v1[0] = i + 1;

        for (j, t) in tar.chars().enumerate() {
            let mut min_cost = v0[j + 1] + 1;
            min_cost = min(min_cost, v1[j] + 1);
            min_cost = min(min_cost, v0[j] + if s == t { 0 } else { 1 });
            v1[j + 1] = min_cost;
        }

        std::mem::swap(&mut v0, &mut v1);
    }

    v0[n - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lev_dist() {
        assert_eq!(lev_dist("saturday", "sunday"), 3);
        assert_eq!(lev_dist("kitten", "sitting"), 3);
        assert_eq!(lev_dist("word", "world"), 1);
        assert_eq!(lev_dist("hello", "hello"), 0);
    }
}
