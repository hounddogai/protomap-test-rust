use super::*;

use std::collections::BTreeMap;

use crate::silence::fallback_silence_target as silence_fallback_silence_target;

pub fn summarize_tiered_silence_cycles_for_exception(
    items: &[silence_fallback_silence_target::TieredSilenceCycle],
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

pub fn derive_alert_exceptions_from_cycle(
    items: &[silence_fallback_silence_target::TieredSilenceCycle],
) -> Vec<AlertException> {
    let mut out: Vec<AlertException> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_exception(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_alert_exception(item: &AlertException) -> String {
    silence_fallback_silence_target::regional_silence_exception_bucket(item.adjusted_volume())
}
