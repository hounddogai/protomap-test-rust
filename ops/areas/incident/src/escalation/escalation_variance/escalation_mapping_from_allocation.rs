use super::*;

use std::collections::BTreeMap;

use crate::page::page_template as page_page_template;

pub fn summarize_regional_page_allocations_for_mapping(
    items: &[page_page_template::RegionalPageAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_escalation_mappings_from_allocation(
    items: &[page_page_template::RegionalPageAllocation],
) -> Vec<EscalationMapping> {
    let mut out: Vec<EscalationMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_escalation_mapping(item: &EscalationMapping) -> String {
    page_page_template::regional_page_allocation_bucket(item.adjusted_load())
}
