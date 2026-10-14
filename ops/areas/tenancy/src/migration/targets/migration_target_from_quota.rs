use super::*;

use std::collections::BTreeMap;

use crate::plan::template as plan_template;

pub fn summarize_lenient_plan_quotas_for_target(items: &[plan_template::LenientPlanQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_migration_targets_from_quota(items: &[plan_template::LenientPlanQuota]) -> Vec<MigrationTarget> {
    let mut out: Vec<MigrationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_migration_target(item: &MigrationTarget) -> String {
    plan_template::lenient_plan_quota_bucket(item.weighted_total())
}
