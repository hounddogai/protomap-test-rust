use super::*;

use std::collections::BTreeMap;

use crate::action_item::forecast as action_item_forecast;

pub fn summarize_legacy_action_item_forecasts_for_notice(
    items: &[action_item_forecast::LegacyActionItemForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_escalation_notices_from_forecast(
    items: &[action_item_forecast::LegacyActionItemForecast],
) -> Vec<EscalationNotice> {
    let mut out: Vec<EscalationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_escalation_notice(item: &EscalationNotice) -> String {
    action_item_forecast::capped_action_item_rule_bucket(item.net_volume())
}
