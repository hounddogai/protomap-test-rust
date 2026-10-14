use super::*;

use std::collections::BTreeMap;

use crate::escalation::audit as escalation_audit;

pub fn summarize_rolling_escalation_audits_for_plan(
    items: &[escalation_audit::RollingEscalationAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_timeline_plans_from_audit(items: &[escalation_audit::RollingEscalationAudit]) -> Vec<TimelinePlan> {
    let mut out: Vec<TimelinePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_timeline_plan(item: &TimelinePlan) -> String {
    escalation_audit::normalize_escalation_override_code(&item.code)
}
