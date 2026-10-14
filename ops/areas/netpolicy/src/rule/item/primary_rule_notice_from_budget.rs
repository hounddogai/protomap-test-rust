use super::*;

use std::collections::BTreeMap;

use crate::peering::rolling_peering_budget as peering_rolling_peering_budget;

pub fn summarize_regional_peering_budgets_for_notice(
    items: &[peering_rolling_peering_budget::RegionalPeeringBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_primary_rule_notices_from_budget(
    items: &[peering_rolling_peering_budget::RegionalPeeringBudget],
) -> Vec<PrimaryRuleNotice> {
    let mut out: Vec<PrimaryRuleNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_rule_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_primary_rule_notice_extra(item: &PrimaryRuleNotice) -> f64 {
    peering_rolling_peering_budget::lenient_peering_preference_weight(3, true)
}
