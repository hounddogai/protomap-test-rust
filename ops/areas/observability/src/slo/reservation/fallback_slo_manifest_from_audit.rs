use super::*;

use std::collections::BTreeMap;

use crate::alert::audit as alert_audit;

pub fn summarize_alert_audits_for_manifest(items: &[alert_audit::AlertAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_fallback_slo_manifests_from_audit(items: &[alert_audit::AlertAudit]) -> Vec<FallbackSloManifest> {
    let mut out: Vec<FallbackSloManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_slo_manifest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_slo_manifest(item: &FallbackSloManifest) -> String {
    alert_audit::alert_audit_bucket(item.projected_value())
}
