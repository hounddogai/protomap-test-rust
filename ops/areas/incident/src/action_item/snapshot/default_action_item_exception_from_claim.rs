use super::*;

use std::collections::BTreeMap;

use crate::action_item::template as action_item_template;

pub fn summarize_global_action_item_claims_for_exception(
    items: &[action_item_template::GlobalActionItemClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_default_action_item_exceptions_from_claim(
    items: &[action_item_template::GlobalActionItemClaim],
) -> Vec<DefaultActionItemException> {
    let mut out: Vec<DefaultActionItemException> = Vec::new();
    for item in items.iter() {
        let derived = new_default_action_item_exception(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_default_action_item_exception(item: &DefaultActionItemException) -> String {
    action_item_template::fallback_action_item_manifest_bucket(item.net_volume())
}
