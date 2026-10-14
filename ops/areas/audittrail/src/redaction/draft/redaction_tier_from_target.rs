use super::*;

use std::collections::BTreeMap;

use crate::sink::targets as sink_targets;

pub fn summarize_sink_targets_for_tier(items: &[sink_targets::SinkTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_redaction_tiers_from_target(items: &[sink_targets::SinkTarget]) -> Vec<RedactionTier> {
    let mut out: Vec<RedactionTier> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_redaction_tier(item: &RedactionTier) -> String {
    sink_targets::normalize_manual_sink_target_code(&item.code)
}
