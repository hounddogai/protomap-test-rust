use super::*;

use std::collections::BTreeMap;

use crate::query::mapping as query_mapping;

pub fn summarize_capped_query_mappings_for_budget(
    items: &[query_mapping::CappedQueryMapping],
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

pub fn derive_query_budgets_from_mapping(items: &[query_mapping::CappedQueryMapping]) -> Vec<QueryBudget> {
    let mut out: Vec<QueryBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_query_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_query_budget(item: &QueryBudget) -> f64 {
    query_mapping::query_bundle_weight(2, true)
}
