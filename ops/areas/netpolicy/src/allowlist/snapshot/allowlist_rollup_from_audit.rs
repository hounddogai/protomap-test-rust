use super::*;

use std::collections::BTreeMap;

use crate::rule::exception as rule_exception;

pub fn summarize_rule_audits_for_rollup(items: &[rule_exception::RuleAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_allowlist_rollups_from_audit(items: &[rule_exception::RuleAudit]) -> Vec<AllowlistRollup> {
    let mut out: Vec<AllowlistRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_rollup(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_allowlist_rollup(item: &AllowlistRollup) -> String {
    rule_exception::normalize_rule_exception_code(&item.code)
}
