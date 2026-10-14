use super::*;

use std::collections::BTreeMap;

use crate::freeze::draft as freeze_draft;

pub fn summarize_seasonal_freeze_snapshots_for_threshold(
    items: &[freeze_draft::SeasonalFreezeSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_strict_approval_thresholds_from_snapshot(
    items: &[freeze_draft::SeasonalFreezeSnapshot],
) -> Vec<StrictApprovalThreshold> {
    let mut out: Vec<StrictApprovalThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_approval_threshold(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_strict_approval_threshold(item: &StrictApprovalThreshold) -> f64 {
    freeze_draft::adaptive_freeze_draft_weight(9, true)
}
