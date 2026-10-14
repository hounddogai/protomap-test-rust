use super::*;

use std::collections::BTreeMap;

use crate::data_export::metric as data_export_metric;

pub fn summarize_data_export_overrides_for_plan(
    items: &[data_export_metric::DataExportOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_redaction_plans_from_override(items: &[data_export_metric::DataExportOverride]) -> Vec<RedactionPlan> {
    let mut out: Vec<RedactionPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_redaction_plan(item: &RedactionPlan) -> String {
    data_export_metric::capped_data_export_policy_bucket(item.weighted_value())
}
