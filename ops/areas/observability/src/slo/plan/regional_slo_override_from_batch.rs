use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_metric as slo_slo_metric;

pub fn summarize_lenient_slo_batches_for_override(items: &[slo_slo_metric::LenientSloBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_regional_slo_overrides_from_batch(items: &[slo_slo_metric::LenientSloBatch]) -> Vec<RegionalSloOverride> {
    let mut out: Vec<RegionalSloOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_slo_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_regional_slo_override(item: &RegionalSloOverride) -> String {
    slo_slo_metric::normalize_lenient_slo_batch_code(&item.code)
}
