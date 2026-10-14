use super::*;

use std::collections::BTreeMap;

use crate::postmortem::mapping as postmortem_mapping;

pub fn summarize_postmortem_cycles_for_template(
    items: &[postmortem_mapping::PostmortemCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_batched_page_templates_from_cycle(
    items: &[postmortem_mapping::PostmortemCycle],
) -> Vec<BatchedPageTemplate> {
    let mut out: Vec<BatchedPageTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_page_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_batched_page_template(item: &BatchedPageTemplate) -> String {
    postmortem_mapping::strict_postmortem_plan_bucket(item.measured_value())
}
