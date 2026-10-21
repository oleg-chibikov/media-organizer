const COPY_COUNTER_LIMIT: u32 = 100;

pub(super) fn find_matching_open_paren(value: &str, close_index: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    if close_index >= bytes.len() || bytes[close_index] != b')' {
        return None;
    }

    let mut depth = 0_u32;
    for index in (0..=close_index).rev() {
        if bytes[index] == b')' {
            depth += 1;
            continue;
        }
        if bytes[index] == b'(' {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

/// Copy counters like `(1)` or `(42)` are not original names.
pub(super) fn is_small_numeric_group(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|ch| ch.is_ascii_digit())
        && value
            .parse::<u32>()
            .is_ok_and(|number| number < COPY_COUNTER_LIMIT)
}

/// Returns the deepest non-counter group, preferring the last one at equal depth.
pub(super) fn parse_most_nested_group(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut stack: Vec<usize> = Vec::new();
    let mut best: Option<(usize, usize, String)> = None;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'(' {
            stack.push(index);
            continue;
        }

        if *byte != b')' {
            continue;
        }

        let Some(start) = stack.pop() else {
            continue;
        };

        let depth = stack.len() + 1;
        let inner = value[start + 1..index].trim();

        if inner.is_empty() || is_small_numeric_group(inner) {
            continue;
        }

        match &best {
            Some((best_depth, best_index, _))
                if depth < *best_depth || (depth == *best_depth && index <= *best_index) => {}
            _ => {
                best = Some((depth, index, inner.to_string()));
            }
        }
    }

    best.map(|(_, _, inner)| inner)
}
