use super::*;

use std::collections::BTreeMap;

use crate::rollout::batch as rollout_batch;

pub fn summarize_rollout_plans_for_audit(items: &[rollout_batch::RolloutPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_legacy_freeze_audits_from_plan(items: &[rollout_batch::RolloutPlan]) -> Vec<LegacyFreezeAudit> {
    let mut out: Vec<LegacyFreezeAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_freeze_audit(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_freeze_audit(item: &LegacyFreezeAudit) -> String {
    rollout_batch::rollout_plan_bucket(item.net_exposure())
}
