use super::*;

use std::collections::BTreeMap;

use crate::probe::mapping as probe_mapping;

pub fn summarize_rolling_probe_mappings_for_record(
    items: &[probe_mapping::RollingProbeMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_fallback_alert_records_from_mapping(
    items: &[probe_mapping::RollingProbeMapping],
) -> Vec<FallbackAlertRecord> {
    let mut out: Vec<FallbackAlertRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_alert_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_alert_record(item: &FallbackAlertRecord) -> f64 {
    probe_mapping::global_probe_entry_weight(8, true)
}
