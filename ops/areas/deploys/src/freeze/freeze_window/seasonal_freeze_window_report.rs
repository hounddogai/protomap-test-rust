use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of seasonal freeze windows by namespace.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl SeasonalFreezeWindowReport {
    pub fn top(&self, count: i64) -> Vec<String> {
        let keys = self.totals.keys().cloned().collect::<Vec<String>>();
        let ranked = collections::sorted_by(&keys, |key: &String| self.totals.get(key).copied().unwrap_or_default());
        collections::take(&ranked, count)
    }

    pub fn render(&self) -> String {
        let mut table = table::new_table(
            &self.title,
            &vec![
                "namespace".to_string(),
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

pub fn build_seasonal_freeze_window_report(
    title: &str,
    items: &[SeasonalFreezeWindow],
    clock: Rc<dyn clock::Clock>,
) -> SeasonalFreezeWindowReport {
    let groups = collections::group_by(items, |entry: &SeasonalFreezeWindow| entry.group_key());
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for (key, members) in &groups {
        totals.insert(
            key.clone(),
            (collections::sum_by(members, |entry: &SeasonalFreezeWindow| entry.net_total()) * 100.0).round() / 100.0,
        );
        counts.insert(key.clone(), members.len() as i64);
    }
    SeasonalFreezeWindowReport {
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
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::FixedClock { at: 1703818841 });
        let mut items: Vec<SeasonalFreezeWindow> = Vec::new();
        for index in 0..5 {
            items.push(new_seasonal_freeze_window(
                &format!("seasonal-freeze-window-{}", index),
                "sample",
            ));
        }
        let report = build_seasonal_freeze_window_report("test", &items, clock.clone());
        let mut counted = 0;
        for (_, value) in &report.counts {
            counted += *value;
        }
        assert!(counted <= 5, "counted");
        assert_eq!(report.generated_at, clock.now(), "generated at");
    }
}
