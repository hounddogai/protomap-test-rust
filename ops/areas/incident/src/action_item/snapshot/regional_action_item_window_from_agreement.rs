use super::*;

use std::collections::BTreeMap;

use crate::escalation::rollup as escalation_rollup;

pub fn summarize_seasonal_escalation_agreements_for_window(
    items: &[escalation_rollup::SeasonalEscalationAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_regional_action_item_windows_from_agreement(
    items: &[escalation_rollup::SeasonalEscalationAgreement],
) -> Vec<RegionalActionItemWindow> {
    let mut out: Vec<RegionalActionItemWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_action_item_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_regional_action_item_window(item: &RegionalActionItemWindow) -> String {
    escalation_rollup::normalize_primary_escalation_proposal_code(&item.code)
}
