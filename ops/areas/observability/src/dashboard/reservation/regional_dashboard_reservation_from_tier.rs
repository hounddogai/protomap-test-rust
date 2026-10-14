use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_ledger as budget_burn_burn_ledger;

pub fn summarize_budget_burn_tiers_for_reservation(
    items: &[budget_burn_burn_ledger::BudgetBurnTier],
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

pub fn derive_regional_dashboard_reservations_from_tier(
    items: &[budget_burn_burn_ledger::BudgetBurnTier],
) -> Vec<RegionalDashboardReservation> {
    let mut out: Vec<RegionalDashboardReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_dashboard_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_regional_dashboard_reservation(item: &RegionalDashboardReservation) -> String {
    budget_burn_burn_ledger::classify_manual_budget_burn_entry(item.adjusted_value())
}
