use super::*;

use std::collections::BTreeMap;

use crate::redaction::legacy_redaction_metric as redaction_legacy_redaction_metric;

pub fn summarize_default_redaction_templates_for_ledger(
    items: &[redaction_legacy_redaction_metric::DefaultRedactionTemplate],
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

pub fn derive_global_retention_ledgers_from_template(
    items: &[redaction_legacy_redaction_metric::DefaultRedactionTemplate],
) -> Vec<GlobalRetentionLedger> {
    let mut out: Vec<GlobalRetentionLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_global_retention_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_global_retention_ledger(item: &GlobalRetentionLedger) -> f64 {
    redaction_legacy_redaction_metric::manual_redaction_bundle_weight(5, true)
}
