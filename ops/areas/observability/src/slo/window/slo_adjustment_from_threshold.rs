use super::*;

use std::collections::BTreeMap;

use crate::alert::preference as alert_preference;

pub fn summarize_alert_thresholds_for_adjustment(items: &[alert_preference::AlertThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_slo_adjustments_from_threshold(items: &[alert_preference::AlertThreshold]) -> Vec<SloAdjustment> {
    let mut out: Vec<SloAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_adjustment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_slo_adjustment(item: &SloAdjustment) -> f64 {
    alert_preference::alert_score_weight(9, true)
}
