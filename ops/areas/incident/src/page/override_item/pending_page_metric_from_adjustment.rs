use super::*;

use std::collections::BTreeMap;

use crate::page::reservation as page_reservation;

pub fn summarize_page_adjustments_for_metric(items: &[page_reservation::PageAdjustment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_pending_page_metrics_from_adjustment(
    items: &[page_reservation::PageAdjustment],
) -> Vec<PendingPageMetric> {
    let mut out: Vec<PendingPageMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_page_metric(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_pending_page_metric(item: &PendingPageMetric) -> String {
    page_reservation::page_variance_bucket(item.measured_value())
}
