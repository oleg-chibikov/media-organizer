use std::path::{Component, Path};

/// A month folder of the dated layout, `<root>/<year>/<month>`, kept as named on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MonthFolder {
    pub(crate) year: i32,
    pub(crate) name: String,
}

impl MonthFolder {
    /// The month folder `selected` is, or a day folder of: `<year>/<month>[/<day>]` under
    /// `root`, as `2026/5`, `2026/05` or `2026/05/12`. Anything else is `None`.
    pub(crate) fn of(root: &Path, selected: &Path) -> Option<Self> {
        let parts: Vec<&str> = selected
            .strip_prefix(root)
            .ok()?
            .components()
            .map(|part| match part {
                Component::Normal(name) => name.to_str(),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let (year, month, day) = match parts.as_slice() {
            [year, month] => (*year, *month, None),
            [year, month, day] => (*year, *month, Some(*day)),
            _ => return None,
        };
        let is_year = year.len() == 4 && number_in(year, 1..=9999);
        let is_day = day.is_none_or(|day| day.len() <= 2 && number_in(day, 1..=31));
        let is_month = month.len() <= 2 && number_in(month, 1..=12);
        if !(is_year && is_month && is_day) {
            return None;
        }
        Some(Self {
            year: year.parse().ok()?,
            name: month.to_owned(),
        })
    }
}

fn number_in(text: &str, range: std::ops::RangeInclusive<u32>) -> bool {
    text.bytes().all(|byte| byte.is_ascii_digit())
        && text.parse().is_ok_and(|value| range.contains(&value))
}
