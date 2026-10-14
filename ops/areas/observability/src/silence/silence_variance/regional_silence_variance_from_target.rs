use super::*;

use std::collections::BTreeMap;

use crate::silence::audit as silence_audit;

pub fn summarize_tiered_silence_targets_for_variance(
    items: &[silence_audit::TieredSilenceTarget],
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

pub fn derive_regional_silence_variances_from_target(
    items: &[silence_audit::TieredSilenceTarget],
) -> Vec<RegionalSilenceVariance> {
    let mut out: Vec<RegionalSilenceVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_silence_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_silence_variance(item: &RegionalSilenceVariance) -> String {
    silence_audit::rolling_silence_audit_bucket(item.effective_total())
}
