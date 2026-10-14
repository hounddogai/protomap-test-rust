use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rule as postmortem_rule;

pub fn summarize_tiered_postmortem_cycles_for_record(
    items: &[postmortem_rule::TieredPostmortemCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_tiered_runbook_records_from_cycle(
    items: &[postmortem_rule::TieredPostmortemCycle],
) -> Vec<TieredRunbookRecord> {
    let mut out: Vec<TieredRunbookRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_runbook_record(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_tiered_runbook_record(item: &TieredRunbookRecord) -> f64 {
    postmortem_rule::primary_postmortem_rule_weight(4, true)
}
