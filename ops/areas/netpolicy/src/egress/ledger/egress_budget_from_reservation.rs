use super::*;

use std::collections::BTreeMap;

use crate::egress::reservation as egress_reservation;

pub fn summarize_egress_reservations_for_budget(
    items: &[egress_reservation::EgressReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_egress_budgets_from_reservation(items: &[egress_reservation::EgressReservation]) -> Vec<EgressBudget> {
    let mut out: Vec<EgressBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_egress_budget(item: &EgressBudget) -> String {
    egress_reservation::egress_agreement_bucket(item.weighted_total())
}
