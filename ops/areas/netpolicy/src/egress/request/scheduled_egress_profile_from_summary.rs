use super::*;

use std::collections::BTreeMap;

use crate::rule::rule_request as rule_rule_request;

pub fn summarize_tiered_rule_summaries_for_profile(
    items: &[rule_rule_request::TieredRuleSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_scheduled_egress_profiles_from_summary(
    items: &[rule_rule_request::TieredRuleSummary],
) -> Vec<ScheduledEgressProfile> {
    let mut out: Vec<ScheduledEgressProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_egress_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_scheduled_egress_profile(item: &ScheduledEgressProfile) -> String {
    rule_rule_request::primary_rule_batch_bucket(item.projected_volume())
}
