use super::*;

use std::collections::BTreeMap;

use crate::isolation::segment as isolation_segment;

pub fn summarize_global_isolation_scores_for_adjustment(
    items: &[isolation_segment::GlobalIsolationScore],
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

pub fn derive_lenient_suspension_adjustments_from_score(
    items: &[isolation_segment::GlobalIsolationScore],
) -> Vec<LenientSuspensionAdjustment> {
    let mut out: Vec<LenientSuspensionAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_suspension_adjustment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_suspension_adjustment(item: &LenientSuspensionAdjustment) -> String {
    isolation_segment::normalize_default_isolation_segment_code(&item.code)
}
