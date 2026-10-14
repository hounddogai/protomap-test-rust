use super::*;

use std::collections::BTreeMap;

use crate::isolation::exception as isolation_exception;

pub fn summarize_isolation_mappings_for_score(
    items: &[isolation_exception::IsolationMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_scheduled_tenant_scores_from_mapping(
    items: &[isolation_exception::IsolationMapping],
) -> Vec<ScheduledTenantScore> {
    let mut out: Vec<ScheduledTenantScore> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_tenant_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_scheduled_tenant_score(item: &ScheduledTenantScore) -> String {
    isolation_exception::classify_scheduled_isolation_exception(item.effective_volume())
}
