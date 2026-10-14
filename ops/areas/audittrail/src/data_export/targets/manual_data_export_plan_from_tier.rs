use super::*;

use std::collections::BTreeMap;

use crate::data_export::export_plan as data_export_export_plan;

pub fn summarize_pending_data_export_tiers_for_plan(
    items: &[data_export_export_plan::PendingDataExportTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_manual_data_export_plans_from_tier(
    items: &[data_export_export_plan::PendingDataExportTier],
) -> Vec<ManualDataExportPlan> {
    let mut out: Vec<ManualDataExportPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_data_export_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_manual_data_export_plan(item: &ManualDataExportPlan) -> String {
    data_export_export_plan::capped_data_export_plan_bucket(item.measured_value())
}
