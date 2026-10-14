use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::entry as budget_burn_entry;

pub fn summarize_strict_budget_burn_templates_for_bundle(
    items: &[budget_burn_entry::StrictBudgetBurnTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_strict_alert_bundles_from_template(
    items: &[budget_burn_entry::StrictBudgetBurnTemplate],
) -> Vec<StrictAlertBundle> {
    let mut out: Vec<StrictAlertBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_alert_bundle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_strict_alert_bundle(item: &StrictAlertBundle) -> f64 {
    budget_burn_entry::pending_budget_burn_record_weight(6, true)
}
