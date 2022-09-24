use std::cmp::min;

pub fn lev_dist(src: &str, tar: &str) -> usize {
    let m = src.len() + 1;
    let n = tar.len() + 1;
    let mut d = vec![vec![0usize; n]; m];

    for i in 1..m {
        d[i][0] = i;
    }

    for j in 1..n {
        d[0][j] = j;
    }

    for (i, s) in src.chars().enumerate() {
        for (j, t) in tar.chars().enumerate() {
            let mut min_cost = d[i][j + 1] + 1;
            min_cost = min(min_cost, d[i + 1][j] + 1);
            min_cost = min(min_cost, d[i][j] + if s == t { 0 } else { 1 });
            d[i + 1][j + 1] = min_cost;
        }
    }

    d[m - 1][n - 1]
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
