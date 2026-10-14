use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_target as slo_slo_target;

pub fn summarize_lenient_slo_targets_for_assignment(
    items: &[slo_slo_target::LenientSloTarget],
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

pub fn derive_adaptive_silence_assignments_from_target(
    items: &[slo_slo_target::LenientSloTarget],
) -> Vec<AdaptiveSilenceAssignment> {
    let mut out: Vec<AdaptiveSilenceAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_silence_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_adaptive_silence_assignment(item: &AdaptiveSilenceAssignment) -> f64 {
    slo_slo_target::seasonal_slo_manifest_weight(8, true)
}
