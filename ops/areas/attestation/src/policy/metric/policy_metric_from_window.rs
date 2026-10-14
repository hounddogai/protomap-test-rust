use super::*;

use std::collections::BTreeMap;

use crate::quote::window as quote_window;

pub fn summarize_quote_windows_for_metric(items: &[quote_window::QuoteWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_policy_metrics_from_window(items: &[quote_window::QuoteWindow]) -> Vec<PolicyMetric> {
    let mut out: Vec<PolicyMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_policy_metric(item: &PolicyMetric) -> String {
    quote_window::normalize_quote_window_code(&item.code)
}
