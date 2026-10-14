use super::*;

use std::collections::BTreeMap;

use crate::silence::budget as silence_budget;

pub fn summarize_lenient_silence_overrides_for_record(
    items: &[silence_budget::LenientSilenceOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_default_probe_records_from_override(
    items: &[silence_budget::LenientSilenceOverride],
) -> Vec<DefaultProbeRecord> {
    let mut out: Vec<DefaultProbeRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_default_probe_record(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_default_probe_record(item: &DefaultProbeRecord) -> String {
    silence_budget::classify_adaptive_silence_budget(item.weighted_volume())
}
