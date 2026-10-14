use super::*;

use std::collections::BTreeMap;

use crate::escalation::rollup as escalation_rollup;

pub fn summarize_seasonal_escalation_agreements_for_cycle(
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

pub fn derive_tiered_postmortem_cycles_from_agreement(
    items: &[escalation_rollup::SeasonalEscalationAgreement],
) -> Vec<TieredPostmortemCycle> {
    let mut out: Vec<TieredPostmortemCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_postmortem_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_tiered_postmortem_cycle(item: &TieredPostmortemCycle) -> String {
    escalation_rollup::normalize_primary_escalation_proposal_code(&item.code)
}
