use super::*;

use std::collections::BTreeMap;

use crate::rule::digest as rule_digest;

pub fn summarize_rule_records_for_bundle(items: &[rule_digest::RuleRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_allowlist_bundles_from_record(items: &[rule_digest::RuleRecord]) -> Vec<StrictAllowlistBundle> {
    let mut out: Vec<StrictAllowlistBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_allowlist_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_strict_allowlist_bundle(item: &StrictAllowlistBundle) -> String {
    rule_digest::normalize_regional_rule_digest_code(&item.code)
}
