use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_audit as alert_alert_audit;

pub fn summarize_tiered_alert_manifests_for_ledger(
    items: &[alert_alert_audit::TieredAlertManifest],
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

pub fn derive_probe_ledgers_from_manifest(items: &[alert_alert_audit::TieredAlertManifest]) -> Vec<ProbeLedger> {
    let mut out: Vec<ProbeLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_ledger(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_probe_ledger(item: &ProbeLedger) -> String {
    alert_alert_audit::tiered_alert_manifest_bucket(item.projected_exposure())
}
