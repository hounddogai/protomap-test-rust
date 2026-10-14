use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_score as delegation_delegation_score;

pub fn summarize_rolling_delegation_templates_for_schedule(
    items: &[delegation_delegation_score::RollingDelegationTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_delegation_schedules_from_template(
    items: &[delegation_delegation_score::RollingDelegationTemplate],
) -> Vec<DelegationSchedule> {
    let mut out: Vec<DelegationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_delegation_schedule_extra(item: &DelegationSchedule) -> String {
    delegation_delegation_score::classify_delegation_milestone(item.net_value())
}
