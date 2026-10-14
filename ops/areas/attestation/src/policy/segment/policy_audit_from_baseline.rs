use super::*;

use std::collections::BTreeMap;

use crate::quote::group as quote_group;

pub fn summarize_strict_quote_baselines_for_audit(items: &[quote_group::StrictQuoteBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_policy_audits_from_baseline(items: &[quote_group::StrictQuoteBaseline]) -> Vec<PolicyAudit> {
    let mut out: Vec<PolicyAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_audit(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_policy_audit(item: &PolicyAudit) -> String {
    quote_group::classify_quote_metric(item.effective_total())
}
