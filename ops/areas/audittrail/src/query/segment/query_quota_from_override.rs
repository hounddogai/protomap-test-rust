use super::*;

use std::collections::BTreeMap;

use crate::retention::override_item as retention_override_item;

pub fn summarize_retention_overrides_for_quota(
    items: &[retention_override_item::RetentionOverride],
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

pub fn derive_query_quotas_from_override(items: &[retention_override_item::RetentionOverride]) -> Vec<QueryQuota> {
    let mut out: Vec<QueryQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_query_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_query_quota(item: &QueryQuota) -> String {
    retention_override_item::classify_batched_retention_override(item.projected_load())
}
