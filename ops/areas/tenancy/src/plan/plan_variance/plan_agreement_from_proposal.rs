use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_tier as isolation_isolation_tier;

pub fn summarize_manual_isolation_proposals_for_agreement(
    items: &[isolation_isolation_tier::ManualIsolationProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_plan_agreements_from_proposal(
    items: &[isolation_isolation_tier::ManualIsolationProposal],
) -> Vec<PlanAgreement> {
    let mut out: Vec<PlanAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_plan_agreement_extra(item: &PlanAgreement) -> String {
    isolation_isolation_tier::regional_isolation_tier_bucket(item.measured_value())
}
