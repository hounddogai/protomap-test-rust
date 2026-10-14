use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of postmortem baselines by scope.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl PostmortemBaselineReport {
    pub fn top(&self, count: i64) -> Vec<String> {
        let keys = self.totals.keys().cloned().collect::<Vec<String>>();
        let ranked = collections::sorted_by(&keys, |key: &String| self.totals.get(key).copied().unwrap_or_default());
        collections::take(&ranked, count)
    }

    pub fn render(&self) -> String {
        let mut table = table::new_table(
            &self.title,
            &vec![
                "scope".to_string(),
                "count".to_string(),
                "total".to_string(),
                "share".to_string(),
            ],
        );
        let mut grand = 0.0;
        for (_, total) in &self.totals {
            grand += *total;
        }
        for (key, total) in &self.totals {
            if table
                .add_row(&vec![
                    key.clone(),
                    self.counts.get(key).copied().unwrap_or_default().to_string(),
                    format!("{:.2}", *total),
                    format!("{}%", numbers::percent(*total, grand)),
                ])
                .is_err()
            {
                continue;
            }
        }
        table.render()
    }
}

pub fn build_postmortem_baseline_report(
    title: &str,
    items: &[PostmortemBaseline],
    clock: Rc<dyn clock::Clock>,
) -> PostmortemBaselineReport {
    let groups = collections::group_by(items, |entry: &PostmortemBaseline| entry.group_key());
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for (key, members) in &groups {
        totals.insert(
            key.clone(),
            (collections::sum_by(members, |entry: &PostmortemBaseline| entry.effective_value()) * 100.0).round()
                / 100.0,
        );
        counts.insert(key.clone(), members.len() as i64);
    }
    PostmortemBaselineReport {
        title: title.to_string(),
        totals,
        counts,
        generated_at: clock.now(),
    }
}
