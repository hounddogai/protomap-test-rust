use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_override as retention_retention_override;

pub fn summarize_regional_retention_notices_for_plan(
    items: &[retention_retention_override::RegionalRetentionNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_sink_plans_from_notice(
    items: &[retention_retention_override::RegionalRetentionNotice],
) -> Vec<PrimarySinkPlan> {
    let mut out: Vec<PrimarySinkPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_sink_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_primary_sink_plan(item: &PrimarySinkPlan) -> String {
    retention_retention_override::regional_retention_notice_bucket(item.adjusted_value())
}
