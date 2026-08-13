use std::collections::BTreeSet;

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakSummary {
    pub current: u32,
    pub longest: u32,
}

/// Calculate current and longest activity streaks from already-localized dates.
///
/// The current streak remains active when the most recent activity is today or
/// yesterday. Older activity is historical and therefore has a current streak
/// of zero.
pub fn calculate_streaks<I>(activity_dates: I, today: NaiveDate) -> StreakSummary
where
    I: IntoIterator<Item = NaiveDate>,
{
    let dates = activity_dates
        .into_iter()
        .filter(|date| *date <= today)
        .collect::<BTreeSet<_>>();

    let mut longest = 0_u32;
    let mut run = 0_u32;
    let mut previous = None;

    for date in &dates {
        run = match previous {
            Some(previous_date) if *date == previous_date + Duration::days(1) => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        previous = Some(*date);
    }

    let Some(last) = dates.last().copied() else {
        return StreakSummary::default();
    };
    if last < today - Duration::days(1) {
        return StreakSummary {
            current: 0,
            longest,
        };
    }

    let mut current = 1_u32;
    let mut cursor = last;
    while let Some(previous_date) = cursor.pred_opt() {
        if !dates.contains(&previous_date) {
            break;
        }
        current += 1;
        cursor = previous_date;
    }

    StreakSummary { current, longest }
}
