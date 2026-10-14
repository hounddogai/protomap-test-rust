use super::*;

use std::collections::BTreeMap;

use crate::quote::manual_quote_plan as quote_manual_quote_plan;

pub fn summarize_default_quote_plans_for_assignment(
    items: &[quote_manual_quote_plan::DefaultQuotePlan],
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

pub fn derive_verifier_assignments_from_plan(
    items: &[quote_manual_quote_plan::DefaultQuotePlan],
) -> Vec<VerifierAssignment> {
    let mut out: Vec<VerifierAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_assignment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_verifier_assignment(item: &VerifierAssignment) -> f64 {
    quote_manual_quote_plan::legacy_quote_group_weight(3, true)
}
