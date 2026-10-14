use super::*;

use std::collections::BTreeMap;

use crate::binding::audit as binding_audit;

pub fn summarize_binding_audits_for_notice(items: &[binding_audit::BindingAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_fallback_token_notices_from_audit(items: &[binding_audit::BindingAudit]) -> Vec<FallbackTokenNotice> {
    let mut out: Vec<FallbackTokenNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_token_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_fallback_token_notice(item: &FallbackTokenNotice) -> String {
    binding_audit::legacy_binding_slot_bucket(item.measured_value())
}
