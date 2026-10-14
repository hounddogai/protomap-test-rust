use super::*;

use std::collections::BTreeMap;

use crate::probe::claim as probe_claim;

pub fn summarize_regional_probe_claims_for_adjustment(
    items: &[probe_claim::RegionalProbeClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_default_slo_adjustments_from_claim(
    items: &[probe_claim::RegionalProbeClaim],
) -> Vec<DefaultSloAdjustment> {
    let mut out: Vec<DefaultSloAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_slo_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_default_slo_adjustment(item: &DefaultSloAdjustment) -> f64 {
    probe_claim::probe_summary_weight(3, true)
}
