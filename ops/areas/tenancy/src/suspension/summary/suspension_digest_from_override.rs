use super::*;

use std::collections::BTreeMap;

use crate::plan::item as plan_item;

pub fn summarize_plan_overrides_for_digest(items: &[plan_item::PlanOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_suspension_digests_from_override(items: &[plan_item::PlanOverride]) -> Vec<SuspensionDigest> {
    let mut out: Vec<SuspensionDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_suspension_digest(item: &SuspensionDigest) -> String {
    plan_item::plan_item_bucket(item.effective_value())
}
