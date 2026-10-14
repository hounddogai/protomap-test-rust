use super::*;

use std::collections::BTreeMap;

use crate::runbook::record_item as runbook_record_item;

pub fn summarize_tiered_runbook_records_for_plan(
    items: &[runbook_record_item::TieredRunbookRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_legacy_postmortem_plans_from_record(
    items: &[runbook_record_item::TieredRunbookRecord],
) -> Vec<LegacyPostmortemPlan> {
    let mut out: Vec<LegacyPostmortemPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_postmortem_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_legacy_postmortem_plan(item: &LegacyPostmortemPlan) -> f64 {
    runbook_record_item::tiered_runbook_record_weight(5, true)
}
