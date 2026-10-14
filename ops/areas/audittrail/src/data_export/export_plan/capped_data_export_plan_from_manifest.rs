use super::*;

use std::collections::BTreeMap;

use crate::query::adaptive_query_manifest as query_adaptive_query_manifest;

pub fn summarize_tiered_query_manifests_for_plan(
    items: &[query_adaptive_query_manifest::TieredQueryManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_capped_data_export_plans_from_manifest(
    items: &[query_adaptive_query_manifest::TieredQueryManifest],
) -> Vec<CappedDataExportPlan> {
    let mut out: Vec<CappedDataExportPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_data_export_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_capped_data_export_plan(item: &CappedDataExportPlan) -> f64 {
    query_adaptive_query_manifest::query_plan_weight(8, true)
}
