use super::*;

use std::collections::BTreeMap;

use crate::delegation::audit as delegation_audit;

pub fn summarize_lenient_delegation_policies_for_template(
    items: &[delegation_audit::LenientDelegationPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_rolling_delegation_templates_from_policy(
    items: &[delegation_audit::LenientDelegationPolicy],
) -> Vec<RollingDelegationTemplate> {
    let mut out: Vec<RollingDelegationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_delegation_template(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_rolling_delegation_template(item: &RollingDelegationTemplate) -> String {
    delegation_audit::normalize_delegation_audit_code(&item.code)
}
