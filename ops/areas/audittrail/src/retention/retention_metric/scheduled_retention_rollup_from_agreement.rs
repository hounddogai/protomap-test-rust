use super::*;

use std::collections::BTreeMap;

use crate::redaction::agreement as redaction_agreement;

pub fn summarize_regional_redaction_agreements_for_rollup(
    items: &[redaction_agreement::RegionalRedactionAgreement],
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

pub fn derive_scheduled_retention_rollups_from_agreement(
    items: &[redaction_agreement::RegionalRedactionAgreement],
) -> Vec<ScheduledRetentionRollup> {
    let mut out: Vec<ScheduledRetentionRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_retention_rollup(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_scheduled_retention_rollup(item: &ScheduledRetentionRollup) -> String {
    redaction_agreement::normalize_redaction_assignment_code(&item.code)
}
