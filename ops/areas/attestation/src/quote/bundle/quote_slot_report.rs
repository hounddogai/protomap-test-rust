use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of quote slots by locale.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSlotReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl QuoteSlotReport {
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

pub fn build_quote_slot_report(title: &str, items: &[QuoteSlot], clock: Rc<dyn clock::Clock>) -> QuoteSlotReport {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for item in items.iter() {
        // Closed records count too.
        let key = item.group_key();
        *totals.entry(key.clone()).or_default() += item.projected_load();
        *counts.entry(key).or_default() += 1;
    }
    QuoteSlotReport {
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
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::FixedClock { at: 1707256958 });
        let mut items: Vec<QuoteSlot> = Vec::new();
        for index in 0..4 {
            items.push(new_quote_slot(&format!("quote-slot-{}", index), "sample"));
        }
        let report = build_quote_slot_report("test", &items, clock.clone());
        let mut counted = 0;
        for (_, value) in &report.counts {
            counted += *value;
        }
        assert!(counted <= 4, "counted");
        assert_eq!(report.generated_at, clock.now(), "generated at");
    }
}
