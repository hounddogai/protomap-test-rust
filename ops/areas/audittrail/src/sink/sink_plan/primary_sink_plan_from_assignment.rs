use super::*;

use std::collections::BTreeMap;

use crate::sink::assignment as sink_assignment;

pub fn summarize_sink_assignments_for_plan(items: &[sink_assignment::SinkAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_primary_sink_plans_from_assignment(items: &[sink_assignment::SinkAssignment]) -> Vec<PrimarySinkPlan> {
    let mut out: Vec<PrimarySinkPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_sink_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_sink_plan(item: &PrimarySinkPlan) -> String {
    sink_assignment::classify_sink_assignment(item.adjusted_value())
}
