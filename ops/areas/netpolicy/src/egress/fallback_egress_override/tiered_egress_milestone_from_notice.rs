use super::*;

use std::collections::BTreeMap;

use crate::rule::rule_notice as rule_rule_notice;

pub fn summarize_lenient_rule_notices_for_milestone(
    items: &[rule_rule_notice::LenientRuleNotice],
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

pub fn derive_tiered_egress_milestones_from_notice(
    items: &[rule_rule_notice::LenientRuleNotice],
) -> Vec<TieredEgressMilestone> {
    let mut out: Vec<TieredEgressMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_egress_milestone(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_egress_milestone(item: &TieredEgressMilestone) -> String {
    rule_rule_notice::classify_strict_rule_draft(item.projected_load())
}
