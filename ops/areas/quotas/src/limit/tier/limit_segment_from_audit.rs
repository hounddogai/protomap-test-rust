use super::*;

use std::collections::BTreeMap;

use crate::usage::agreement as usage_agreement;

pub fn summarize_legacy_usage_audits_for_segment(items: &[usage_agreement::LegacyUsageAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_limit_segments_from_audit(items: &[usage_agreement::LegacyUsageAudit]) -> Vec<LimitSegment> {
    let mut out: Vec<LimitSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_segment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_limit_segment(item: &LimitSegment) -> f64 {
    usage_agreement::legacy_usage_audit_weight(1, true)
}
