use super::*;

use std::collections::BTreeMap;

use crate::failover::profile as failover_profile;

pub fn summarize_failover_adjustments_for_audit(
    items: &[failover_profile::FailoverAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_health_check_audits_from_adjustment(
    items: &[failover_profile::FailoverAdjustment],
) -> Vec<HealthCheckAudit> {
    let mut out: Vec<HealthCheckAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_audit(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_health_check_audit(item: &HealthCheckAudit) -> String {
    failover_profile::normalize_failover_score_code(&item.code)
}
