use super::*;

use std::collections::BTreeMap;

use crate::usage::estimate as usage_estimate;

pub fn summarize_usage_audits_for_entry(items: &[usage_estimate::UsageAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_overage_entries_from_audit(items: &[usage_estimate::UsageAudit]) -> Vec<OverageEntry> {
    let mut out: Vec<OverageEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_entry(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_overage_entry(item: &OverageEntry) -> f64 {
    usage_estimate::usage_estimate_weight(1, true)
}
