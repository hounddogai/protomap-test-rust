use super::*;

use std::collections::BTreeMap;

use crate::identity::plan as identity_plan;

pub fn summarize_global_identity_plans_for_rule(items: &[identity_plan::GlobalIdentityPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_credential_rules_from_plan(
    items: &[identity_plan::GlobalIdentityPlan],
) -> Vec<LenientCredentialRule> {
    let mut out: Vec<LenientCredentialRule> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_credential_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_lenient_credential_rule(item: &LenientCredentialRule) -> String {
    identity_plan::classify_global_identity_plan(item.effective_total())
}
