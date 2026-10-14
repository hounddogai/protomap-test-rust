use super::*;

use std::collections::BTreeMap;

use crate::isolation::plan as isolation_plan;

pub fn summarize_isolation_groups_for_policy(items: &[isolation_plan::IsolationGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_suspension_policies_from_group(items: &[isolation_plan::IsolationGroup]) -> Vec<SuspensionPolicy> {
    let mut out: Vec<SuspensionPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_policy(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_suspension_policy(item: &SuspensionPolicy) -> String {
    isolation_plan::isolation_plan_bucket(item.measured_value())
}
