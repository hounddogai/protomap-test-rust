use super::*;

use std::collections::BTreeMap;

use crate::health_check::template as health_check_template;

pub fn summarize_capped_health_check_assignments_for_agreement(
    items: &[health_check_template::CappedHealthCheckAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_ttl_policy_agreements_from_assignment(
    items: &[health_check_template::CappedHealthCheckAssignment],
) -> Vec<RollingTtlPolicyAgreement> {
    let mut out: Vec<RollingTtlPolicyAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_ttl_policy_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_ttl_policy_agreement(item: &RollingTtlPolicyAgreement) -> f64 {
    health_check_template::strict_health_check_target_weight(6, true)
}
