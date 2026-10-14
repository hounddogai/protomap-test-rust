use super::*;

use std::collections::BTreeMap;

use crate::runbook::ledger as runbook_ledger;

pub fn summarize_runbook_tiers_for_quota(items: &[runbook_ledger::RunbookTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_postmortem_quotas_from_tier(items: &[runbook_ledger::RunbookTier]) -> Vec<PostmortemQuota> {
    let mut out: Vec<PostmortemQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_quota(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_postmortem_quota(item: &PostmortemQuota) -> String {
    runbook_ledger::runbook_quota_bucket(item.measured_value())
}
