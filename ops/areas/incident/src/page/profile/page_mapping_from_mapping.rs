use super::*;

use std::collections::BTreeMap;

use crate::action_item::exception as action_item_exception;

pub fn summarize_tiered_action_item_mappings_for_mapping(
    items: &[action_item_exception::TieredActionItemMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_page_mappings_from_mapping(items: &[action_item_exception::TieredActionItemMapping]) -> Vec<PageMapping> {
    let mut out: Vec<PageMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_page_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_page_mapping(item: &PageMapping) -> String {
    action_item_exception::classify_action_item_budget(item.measured_value())
}
