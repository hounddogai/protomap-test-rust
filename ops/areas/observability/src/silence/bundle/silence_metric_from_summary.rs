use super::*;

use std::collections::BTreeMap;

use crate::dashboard::schedule as dashboard_schedule;

pub fn summarize_dashboard_summaries_for_metric(
    items: &[dashboard_schedule::DashboardSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_silence_metrics_from_summary(items: &[dashboard_schedule::DashboardSummary]) -> Vec<SilenceMetric> {
    let mut out: Vec<SilenceMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_metric(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_silence_metric(item: &SilenceMetric) -> String {
    dashboard_schedule::dashboard_schedule_bucket(item.weighted_value())
}
