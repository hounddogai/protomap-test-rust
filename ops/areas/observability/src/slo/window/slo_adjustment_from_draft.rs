use super::*;

use std::collections::BTreeMap;

use crate::slo::metric as slo_metric;

pub fn summarize_primary_slo_drafts_for_adjustment(items: &[slo_metric::PrimarySloDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_slo_adjustments_from_draft(items: &[slo_metric::PrimarySloDraft]) -> Vec<SloAdjustment> {
    let mut out: Vec<SloAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_adjustment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_slo_adjustment(item: &SloAdjustment) -> f64 {
    slo_metric::adaptive_slo_tier_weight(6, true)
}
