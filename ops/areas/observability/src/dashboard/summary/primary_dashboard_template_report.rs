use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::clock;
use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::table;

/// Totals of primary dashboard templates by title.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateReport {
    pub title: String,
    pub totals: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, i64>,
    pub generated_at: i64,
}

impl PrimaryDashboardTemplateReport {
    pub fn top(&self, count: i64) -> Vec<String> {
        let keys = self.totals.keys().cloned().collect::<Vec<String>>();
        let ranked = collections::sorted_by(&keys, |key: &String| self.totals.get(key).copied().unwrap_or_default());
        collections::take(&ranked, count)
    }

    pub fn render(&self) -> String {
        let mut table = table::new_table(
            &self.title,
            &vec![
                "title".to_string(),
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

pub fn build_primary_dashboard_template_report(
    title: &str,
    items: &[PrimaryDashboardTemplate],
    clock: Rc<dyn clock::Clock>,
) -> PrimaryDashboardTemplateReport {
    let groups = collections::group_by(items, |entry: &PrimaryDashboardTemplate| entry.group_key());
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for (key, members) in &groups {
        totals.insert(
            key.clone(),
            (collections::sum_by(members, |entry: &PrimaryDashboardTemplate| entry.adjusted_value()) * 100.0).round()
                / 100.0,
        );
        counts.insert(key.clone(), members.len() as i64);
    }
    PrimaryDashboardTemplateReport {
        title: title.to_string(),
        totals,
        counts,
        generated_at: clock.now(),
    }
}
