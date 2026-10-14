use super::*;

use std::collections::BTreeMap;

use crate::rule::policy as rule_policy;

pub fn summarize_seasonal_rule_cycles_for_audit(items: &[rule_policy::SeasonalRuleCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_primary_peering_audits_from_cycle(items: &[rule_policy::SeasonalRuleCycle]) -> Vec<PrimaryPeeringAudit> {
    let mut out: Vec<PrimaryPeeringAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_peering_audit(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_primary_peering_audit(item: &PrimaryPeeringAudit) -> String {
    rule_policy::normalize_rule_summary_code(&item.code)
}
