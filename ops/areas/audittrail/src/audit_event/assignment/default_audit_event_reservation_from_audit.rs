use super::*;

use std::collections::BTreeMap;

use crate::retention::audit as retention_audit;

pub fn summarize_primary_retention_audits_for_reservation(
    items: &[retention_audit::PrimaryRetentionAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_default_audit_event_reservations_from_audit(
    items: &[retention_audit::PrimaryRetentionAudit],
) -> Vec<DefaultAuditEventReservation> {
    let mut out: Vec<DefaultAuditEventReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_audit_event_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_default_audit_event_reservation(item: &DefaultAuditEventReservation) -> String {
    retention_audit::normalize_retention_proposal_code(&item.code)
}
