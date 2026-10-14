use super::*;

use std::collections::BTreeMap;

use crate::health_check::audit as health_check_audit;

pub fn summarize_scheduled_health_check_audits_for_summary(
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

pub fn derive_zone_summaries_from_audit(items: &[health_check_audit::ScheduledHealthCheckAudit]) -> Vec<ZoneSummary> {
    let mut out: Vec<ZoneSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_zone_summary(item: &ZoneSummary) -> String {
    health_check_audit::normalize_scheduled_health_check_audit_code(&item.code)
}
