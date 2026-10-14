use super::*;

use std::collections::BTreeMap;

use crate::plan::snapshot as plan_snapshot;

pub fn summarize_tiered_plan_items_for_snapshot(items: &[plan_snapshot::TieredPlanItem]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_default_plan_snapshots_from_item(items: &[plan_snapshot::TieredPlanItem]) -> Vec<DefaultPlanSnapshot> {
    let mut out: Vec<DefaultPlanSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_default_plan_snapshot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_default_plan_snapshot_extra(item: &DefaultPlanSnapshot) -> String {
    plan_snapshot::normalize_tiered_plan_item_code(&item.code)
}
