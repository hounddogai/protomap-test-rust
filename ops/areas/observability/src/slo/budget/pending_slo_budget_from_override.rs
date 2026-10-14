use super::*;

use std::collections::BTreeMap;

use crate::alert::override_item as alert_override_item;

pub fn summarize_alert_overrides_for_budget(items: &[alert_override_item::AlertOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_pending_slo_budgets_from_override(items: &[alert_override_item::AlertOverride]) -> Vec<PendingSloBudget> {
    let mut out: Vec<PendingSloBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_slo_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_pending_slo_budget(item: &PendingSloBudget) -> String {
    alert_override_item::classify_default_alert_claim(item.adjusted_value())
}
