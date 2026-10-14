use super::*;

use std::collections::BTreeMap;

use crate::dns_record::template as dns_record_template;

pub fn summarize_dns_record_reservations_for_agreement(
    items: &[dns_record_template::DnsRecordReservation],
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

pub fn derive_ttl_policy_agreements_from_reservation(
    items: &[dns_record_template::DnsRecordReservation],
) -> Vec<TtlPolicyAgreement> {
    let mut out: Vec<TtlPolicyAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_agreement(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_ttl_policy_agreement(item: &TtlPolicyAgreement) -> String {
    dns_record_template::classify_dns_record_template(item.weighted_exposure())
}
