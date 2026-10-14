use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_preference as alert_alert_preference;

pub fn summarize_manual_alert_schedules_for_summary(
    items: &[alert_alert_preference::ManualAlertSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_alert_summaries_from_schedule(
    items: &[alert_alert_preference::ManualAlertSchedule],
) -> Vec<AlertSummary> {
    let mut out: Vec<AlertSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_alert_summary(item: &AlertSummary) -> String {
    alert_alert_preference::seasonal_alert_record_bucket(item.effective_load())
}
