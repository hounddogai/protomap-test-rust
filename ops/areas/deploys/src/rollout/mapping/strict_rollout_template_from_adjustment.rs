use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_notice as approval_approval_notice;

pub fn summarize_tiered_approval_adjustments_for_template(
    items: &[approval_approval_notice::TieredApprovalAdjustment],
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

pub fn derive_strict_rollout_templates_from_adjustment(
    items: &[approval_approval_notice::TieredApprovalAdjustment],
) -> Vec<StrictRolloutTemplate> {
    let mut out: Vec<StrictRolloutTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_rollout_template(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_rollout_template(item: &StrictRolloutTemplate) -> String {
    approval_approval_notice::capped_approval_notice_bucket(item.weighted_load())
}
