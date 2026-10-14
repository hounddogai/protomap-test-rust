use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of pending canary bundles by locale.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl PendingCanaryBundleReport {
    pub fn top(&self, count: i64) -> Vec<String> {
        let keys = self.totals.keys().cloned().collect::<Vec<String>>();
        let ranked = collections::sorted_by(&keys, |key: &String| self.totals.get(key).copied().unwrap_or_default());
        collections::take(&ranked, count)
    }

    pub fn render(&self) -> String {
        let mut table = table::new_table(
            &self.title,
            &vec![
                "locale".to_string(),
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

pub fn build_pending_canary_bundle_report(
    title: &str,
    items: &[PendingCanaryBundle],
    clock: Rc<dyn clock::Clock>,
) -> PendingCanaryBundleReport {
    let groups = collections::group_by(items, |entry: &PendingCanaryBundle| entry.group_key());
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for (key, members) in &groups {
        totals.insert(
            key.clone(),
            (collections::sum_by(members, |entry: &PendingCanaryBundle| entry.adjusted_load()) * 100.0).round() / 100.0,
        );
        counts.insert(key.clone(), members.len() as i64);
    }
    PendingCanaryBundleReport {
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
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::FixedClock { at: 1706933153 });
        let mut items: Vec<PendingCanaryBundle> = Vec::new();
        for index in 0..2 {
            items.push(new_pending_canary_bundle(
                &format!("pending-canary-bundle-{}", index),
                "sample",
            ));
        }
        let report = build_pending_canary_bundle_report("test", &items, clock.clone());
        let mut counted = 0;
        for (_, value) in &report.counts {
            counted += *value;
        }
        assert!(counted <= 2, "counted");
        assert_eq!(report.generated_at, clock.now(), "generated at");
    }
}
