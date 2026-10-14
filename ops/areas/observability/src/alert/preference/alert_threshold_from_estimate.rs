use super::*;

use std::collections::BTreeMap;

use crate::silence::estimate as silence_estimate;

pub fn summarize_silence_estimates_for_threshold(items: &[silence_estimate::SilenceEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_alert_thresholds_from_estimate(items: &[silence_estimate::SilenceEstimate]) -> Vec<AlertThreshold> {
    let mut out: Vec<AlertThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_alert_threshold(item: &AlertThreshold) -> f64 {
    silence_estimate::silence_estimate_weight(2, true)
}
