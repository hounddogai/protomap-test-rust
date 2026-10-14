use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::notice as ttl_policy_notice;

pub fn summarize_ttl_policy_variances_for_template(
    items: &[ttl_policy_notice::TtlPolicyVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_delegation_templates_from_variance(
    items: &[ttl_policy_notice::TtlPolicyVariance],
) -> Vec<DelegationTemplate> {
    let mut out: Vec<DelegationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_delegation_template(item: &DelegationTemplate) -> f64 {
    ttl_policy_notice::ttl_policy_notice_weight(2, true)
}
