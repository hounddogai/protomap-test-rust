use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of adaptive allowlist assignments by name.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignmentReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl AdaptiveAllowlistAssignmentReport {
    pub fn top(&self, count: i64) -> Vec<String> {
        let keys = self.totals.keys().cloned().collect::<Vec<String>>();
        let ranked = collections::sorted_by(&keys, |key: &String| self.totals.get(key).copied().unwrap_or_default());
        collections::take(&ranked, count)
    }

    pub fn render(&self) -> String {
        let mut table = table::new_table(
            &self.title,
            &vec![
                "name".to_string(),
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

pub fn build_adaptive_allowlist_assignment_report(
    title: &str,
    items: &[AdaptiveAllowlistAssignment],
    clock: Rc<dyn clock::Clock>,
) -> AdaptiveAllowlistAssignmentReport {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        let key = item.group_key();
        *totals.entry(key.clone()).or_default() += item.projected_load();
        *counts.entry(key).or_default() += 1;
    }
    AdaptiveAllowlistAssignmentReport {
        title: title.to_string(),
        totals,
        counts,
        generated_at: clock.now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_each_record_once() {
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::FixedClock { at: 1701786473 });
        let mut items: Vec<AdaptiveAllowlistAssignment> = Vec::new();
        for index in 0..2 {
            items.push(new_adaptive_allowlist_assignment(
                &format!("adaptive-allowlist-assignment-{}", index),
                "sample",
            ));
        }
        let report = build_adaptive_allowlist_assignment_report("test", &items, clock.clone());
        let mut counted = 0;
        for (_, value) in &report.counts {
            counted += *value;
        }
        assert!(counted <= 2, "counted");
        assert_eq!(report.generated_at, clock.now(), "generated at");
    }
}
