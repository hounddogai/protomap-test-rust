use super::*;

use std::collections::BTreeMap;

use crate::health_check::audit as health_check_audit;

pub fn summarize_scheduled_health_check_audits_for_score(
    items: &[health_check_audit::ScheduledHealthCheckAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_ttl_policy_scores_from_audit(
    items: &[health_check_audit::ScheduledHealthCheckAudit],
) -> Vec<TtlPolicyScore> {
    let mut out: Vec<TtlPolicyScore> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_ttl_policy_score(item: &TtlPolicyScore) -> String {
    health_check_audit::normalize_scheduled_health_check_audit_code(&item.code)
}
