use super::*;

use std::collections::BTreeMap;

use crate::flow_log::digest as flow_log_digest;

pub fn summarize_lenient_flow_log_entries_for_agreement(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_tiered_peering_agreements_from_entry(
    items: &[flow_log_digest::LenientFlowLogEntry],
) -> Vec<TieredPeeringAgreement> {
    let mut out: Vec<TieredPeeringAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_peering_agreement(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_peering_agreement(item: &TieredPeeringAgreement) -> String {
    flow_log_digest::normalize_rolling_flow_log_checkpoint_code(&item.code)
}
