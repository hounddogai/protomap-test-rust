use super::*;

use std::collections::BTreeMap;

use crate::credential::plan as credential_plan;

pub fn summarize_manual_credential_plans_for_group(
    items: &[credential_plan::ManualCredentialPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_tiered_binding_groups_from_plan(
    items: &[credential_plan::ManualCredentialPlan],
) -> Vec<TieredBindingGroup> {
    let mut out: Vec<TieredBindingGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_binding_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_tiered_binding_group(item: &TieredBindingGroup) -> String {
    credential_plan::classify_credential_rule(item.effective_load())
}
