use super::*;

use std::collections::BTreeMap;

use crate::suspension::pending_suspension_metric as suspension_pending_suspension_metric;

pub fn summarize_fallback_suspension_budgets_for_adjustment(
    items: &[suspension_pending_suspension_metric::FallbackSuspensionBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_strict_tenant_adjustments_from_budget(
    items: &[suspension_pending_suspension_metric::FallbackSuspensionBudget],
) -> Vec<StrictTenantAdjustment> {
    let mut out: Vec<StrictTenantAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_tenant_adjustment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_strict_tenant_adjustment(item: &StrictTenantAdjustment) -> String {
    suspension_pending_suspension_metric::classify_scheduled_suspension_metric(item.adjusted_value())
}
