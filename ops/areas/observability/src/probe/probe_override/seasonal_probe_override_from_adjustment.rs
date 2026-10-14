use super::*;

use std::collections::BTreeMap;

use crate::slo::adjustment as slo_adjustment;

pub fn summarize_default_slo_adjustments_for_override(
    items: &[slo_adjustment::DefaultSloAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_seasonal_probe_overrides_from_adjustment(
    items: &[slo_adjustment::DefaultSloAdjustment],
) -> Vec<SeasonalProbeOverride> {
    let mut out: Vec<SeasonalProbeOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_probe_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_probe_override(item: &SeasonalProbeOverride) -> f64 {
    slo_adjustment::slo_mapping_weight(7, true)
}
