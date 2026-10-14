use super::*;

use std::collections::BTreeMap;

use crate::dns_record::agreement as dns_record_agreement;

pub fn summarize_legacy_dns_record_forecasts_for_entry(
    items: &[dns_record_agreement::LegacyDnsRecordForecast],
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

pub fn derive_failover_entries_from_forecast(
    items: &[dns_record_agreement::LegacyDnsRecordForecast],
) -> Vec<FailoverEntry> {
    let mut out: Vec<FailoverEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_failover_entry(item: &FailoverEntry) -> String {
    dns_record_agreement::normalize_legacy_dns_record_forecast_code(&item.code)
}
