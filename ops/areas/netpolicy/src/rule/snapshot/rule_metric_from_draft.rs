use super::*;

use std::collections::BTreeMap;

use crate::allowlist::digest as allowlist_digest;

pub fn summarize_allowlist_drafts_for_metric(items: &[allowlist_digest::AllowlistDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rule_metrics_from_draft(items: &[allowlist_digest::AllowlistDraft]) -> Vec<RuleMetric> {
    let mut out: Vec<RuleMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rule_metric(item: &RuleMetric) -> String {
    allowlist_digest::normalize_allowlist_digest_code(&item.code)
}
