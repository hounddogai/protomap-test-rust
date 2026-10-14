use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_reservation as freeze_freeze_reservation;

pub fn summarize_default_freeze_profiles_for_agreement(
    items: &[freeze_freeze_reservation::DefaultFreezeProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_adaptive_approval_agreements_from_profile(
    items: &[freeze_freeze_reservation::DefaultFreezeProfile],
) -> Vec<AdaptiveApprovalAgreement> {
    let mut out: Vec<AdaptiveApprovalAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_approval_agreement(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_adaptive_approval_agreement(item: &AdaptiveApprovalAgreement) -> f64 {
    freeze_freeze_reservation::pending_freeze_reservation_weight(3, true)
}
