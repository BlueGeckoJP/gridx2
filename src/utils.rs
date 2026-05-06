use regex::Regex;
use std::cmp::{Ordering, min};
use std::sync::LazyLock;

static NATURAL_SORT_RE_ALL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)|(\D+)").unwrap());
static NATURAL_SORT_RE_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());

pub fn natural_sort(a: &str, b: &str, descending: bool) -> eyre::Result<Ordering> {
    let a_parts: Vec<&str> = NATURAL_SORT_RE_ALL
        .find_iter(a)
        .map(|m| m.as_str())
        .collect();
    let b_parts: Vec<&str> = NATURAL_SORT_RE_ALL
        .find_iter(b)
        .map(|m| m.as_str())
        .collect();

    for i in 0..min(a_parts.len(), b_parts.len()) {
        let a_part = a_parts[i];
        let b_part = b_parts[i];

        if a_part == b_part {
            continue;
        }

        let order = if NATURAL_SORT_RE_NUM.is_match(a_part) && NATURAL_SORT_RE_NUM.is_match(b_part)
        {
            let a_num = a_part.parse::<isize>().unwrap_or(0);
            let b_num = b_part.parse::<isize>().unwrap_or(0);
            a_num.cmp(&b_num)
        } else if NATURAL_SORT_RE_NUM.is_match(a_part) {
            Ordering::Less
        } else if NATURAL_SORT_RE_NUM.is_match(b_part) {
            Ordering::Greater
        } else {
            a_part.cmp(b_part)
        };

        if order != Ordering::Equal {
            return Ok(match descending {
                true => order.reverse(),
                false => order,
            });
        }
    }

    let order = a_parts.len().cmp(&b_parts.len());
    Ok(match descending {
        true => order.reverse(),
        false => order,
    })
}
