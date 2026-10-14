use super::*;

use std::collections::BTreeMap;

use crate::dashboard::global_dashboard_ledger as dashboard_global_dashboard_ledger;

pub fn summarize_scheduled_dashboard_ledgers_for_schedule(
    items: &[dashboard_global_dashboard_ledger::ScheduledDashboardLedger],
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

pub fn derive_lenient_slo_schedules_from_ledger(
    items: &[dashboard_global_dashboard_ledger::ScheduledDashboardLedger],
) -> Vec<LenientSloSchedule> {
    let mut out: Vec<LenientSloSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_slo_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_slo_schedule(item: &LenientSloSchedule) -> f64 {
    dashboard_global_dashboard_ledger::scheduled_dashboard_ledger_weight(1, true)
}
