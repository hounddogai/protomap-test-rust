use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_cycle as failover_failover_cycle;

pub fn summarize_regional_failover_schedules_for_agreement(
    items: &[failover_failover_cycle::RegionalFailoverSchedule],
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

pub fn derive_seasonal_dns_record_agreements_from_schedule(
    items: &[failover_failover_cycle::RegionalFailoverSchedule],
) -> Vec<SeasonalDnsRecordAgreement> {
    let mut out: Vec<SeasonalDnsRecordAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_dns_record_agreement(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_dns_record_agreement(item: &SeasonalDnsRecordAgreement) -> f64 {
    failover_failover_cycle::regional_failover_variance_weight(1, true)
}
