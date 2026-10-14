//! Generic helpers over lists and maps.

use std::collections::BTreeMap;

/// Returns the items keep accepts, in order.
pub fn filter<T: Clone + std::fmt::Debug>(items: &[T], keep: impl Fn(&T) -> bool) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    for item in items.iter() {
        if keep(item) {
            out.push(item.clone());
        }
    }
    out
}

pub fn count_by<T: Clone + std::fmt::Debug>(items: &[T], key: impl Fn(&T) -> String) -> BTreeMap<String, i64> {
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for item in items.iter() {
        *counts.entry(key(item)).or_default() += 1;
    }
    counts
}

/// Groups items by key, keeping each group in input order.
pub fn group_by<T: Clone + std::fmt::Debug>(items: &[T], key: impl Fn(&T) -> String) -> BTreeMap<String, Vec<T>> {
    let mut groups: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for item in items.iter() {
        groups.entry(key(item)).or_default().push(item.clone());
    }
    groups
}

pub fn first<T: Clone + std::fmt::Debug>(items: &[T], pred: impl Fn(&T) -> bool) -> Option<T> {
    for item in items.iter() {
        if pred(item) {
            return Some(item.clone());
        }
    }
    None
}

pub fn any_match<T: Clone + std::fmt::Debug>(items: &[T], pred: impl Fn(&T) -> bool) -> bool {
    for item in items.iter() {
        if pred(item) {
            return true;
        }
    }
    false
}

pub fn all_match<T: Clone + std::fmt::Debug>(items: &[T], pred: impl Fn(&T) -> bool) -> bool {
    for item in items.iter() {
        if !pred(item) {
            return false;
        }
    }
    true
}

pub fn sum_by<T: Clone + std::fmt::Debug>(items: &[T], measure: impl Fn(&T) -> f64) -> f64 {
    let mut total = 0.0;
    for item in items.iter() {
        total += measure(item);
    }
    total
}

/// Splits items into batches of at most size, the last one shorter.
pub fn chunk<T: Clone + std::fmt::Debug>(items: &[T], size: i64) -> Vec<Vec<T>> {
    let mut chunks: Vec<Vec<T>> = Vec::new();
    if size <= 0 {
        return chunks.clone();
    }
    let mut current: Vec<T> = Vec::new();
    for item in items.iter() {
        current.push(item.clone());
        if (current.len() as i64) == size {
            chunks.push(current.clone());
            current = Vec::new();
        }
    }
    if (current.len() as i64) > 0 {
        chunks.push(current);
    }
    chunks
}

/// Returns a copy of items ordered by score, highest first. The sort is stable.
pub fn sorted_by<T: Clone + std::fmt::Debug>(items: &[T], score: impl Fn(&T) -> f64) -> Vec<T> {
    let mut sorted = items.to_vec();
    for i in 0..sorted.len() as i64 {
        let mut j = i;
        while j > 0 && (score(&sorted[(j - 1) as usize]) < score(&sorted[j as usize])) {
            let swap = sorted[j as usize].clone();
            sorted[j as usize] = sorted[(j - 1) as usize].clone();
            sorted[(j - 1) as usize] = swap;
            j = j - 1;
        }
    }
    sorted
}

/// Drops repeated values, keeping the first of each.
pub fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen: BTreeMap<String, bool> = BTreeMap::new();
    let mut out: Vec<String> = Vec::new();
    for value in values.iter() {
        if seen.contains_key(value) {
            continue;
        }
        seen.insert(value.clone(), true);
        out.push(value.clone());
    }
    out
}

pub fn take<T: Clone + std::fmt::Debug>(items: &[T], count: i64) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    for item in items.iter() {
        if (out.len() as i64) >= count {
            break;
        }
        out.push(item.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_into_batches() {
        let items: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7];
        let chunks = chunk(&items, 3);
        assert_eq!(chunks.len() as i64, 3, "chunks");
        assert_eq!(chunks[2].len() as i64, 1, "last chunk");
    }

    #[test]
    fn filters_and_counts() {
        let items: Vec<i64> = vec![4, 9, 15, 16, 23, 42];
        let even = filter(&items, |n: &i64| *n % 2 == 0);
        assert_eq!(even.len() as i64, 3, "even");
        let counts = count_by(&items, |n: &i64| -> String {
            if *n > 10 {
                return "big".to_string();
            }
            "small".to_string()
        });
        assert_eq!(counts.get("big").copied().unwrap_or_default(), 4, "big");
    }

    #[test]
    fn drops_repeated_strings() {
        assert_eq!(
            unique_strings(&vec!["a".to_string(), "b".to_string(), "a".to_string()]).len() as i64,
            2,
            "unique"
        );
    }
}
