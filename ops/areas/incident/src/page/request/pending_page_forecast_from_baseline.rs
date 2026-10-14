use super::*;

use std::collections::BTreeMap;

use crate::timeline::baseline as timeline_baseline;

pub fn summarize_scheduled_timeline_baselines_for_forecast(
    items: &[timeline_baseline::ScheduledTimelineBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_pending_page_forecasts_from_baseline(
    items: &[timeline_baseline::ScheduledTimelineBaseline],
) -> Vec<PendingPageForecast> {
    let mut out: Vec<PendingPageForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_page_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_page_forecast(item: &PendingPageForecast) -> String {
    timeline_baseline::classify_timeline_audit(item.effective_exposure())
}
