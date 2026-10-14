use super::*;

use std::collections::BTreeMap;

use crate::tenant::estimate as tenant_estimate;

pub fn summarize_scheduled_tenant_scores_for_reservation(
    items: &[tenant_estimate::ScheduledTenantScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_suspension_reservations_from_score(
    items: &[tenant_estimate::ScheduledTenantScore],
) -> Vec<SuspensionReservation> {
    let mut out: Vec<SuspensionReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_reservation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_suspension_reservation(item: &SuspensionReservation) -> String {
    tenant_estimate::classify_scheduled_tenant_score(item.measured_value())
}
