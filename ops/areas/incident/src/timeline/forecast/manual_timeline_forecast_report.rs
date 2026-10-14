use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of manual timeline forecasts by locale.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineForecastReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl ManualTimelineForecastReport {
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

pub fn build_manual_timeline_forecast_report(
    title: &str,
    items: &[ManualTimelineForecast],
    clock: Rc<dyn clock::Clock>,
) -> ManualTimelineForecastReport {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        let key = item.group_key();
        *totals.entry(key.clone()).or_default() += item.weighted_value();
        *counts.entry(key).or_default() += 1;
    }
    ManualTimelineForecastReport {
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
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::FixedClock { at: 1704218674 });
        let mut items: Vec<ManualTimelineForecast> = Vec::new();
        for index in 0..6 {
            items.push(new_manual_timeline_forecast(
                &format!("manual-timeline-forecast-{}", index),
                "sample",
            ));
        }
        let report = build_manual_timeline_forecast_report("test", &items, clock.clone());
        let mut counted = 0;
        for (_, value) in &report.counts {
            counted += *value;
        }
        assert!(counted <= 6, "counted");
        assert_eq!(report.generated_at, clock.now(), "generated at");
    }
}
