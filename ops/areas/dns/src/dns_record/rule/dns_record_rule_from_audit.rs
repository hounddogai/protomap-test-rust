use super::*;

use std::collections::BTreeMap;

use crate::delegation::audit as delegation_audit;

pub fn summarize_delegation_audits_for_rule(items: &[delegation_audit::DelegationAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_dns_record_rules_from_audit(items: &[delegation_audit::DelegationAudit]) -> Vec<DnsRecordRule> {
    let mut out: Vec<DnsRecordRule> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_dns_record_rule(item: &DnsRecordRule) -> String {
    delegation_audit::normalize_delegation_audit_code(&item.code)
}
