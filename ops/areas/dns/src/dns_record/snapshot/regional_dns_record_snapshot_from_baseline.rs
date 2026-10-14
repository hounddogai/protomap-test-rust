use super::*;

use std::collections::BTreeMap;

use crate::health_check::cycle as health_check_cycle;

pub fn summarize_seasonal_health_check_baselines_for_snapshot(
    items: &[health_check_cycle::SeasonalHealthCheckBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_regional_dns_record_snapshots_from_baseline(
    items: &[health_check_cycle::SeasonalHealthCheckBaseline],
) -> Vec<RegionalDnsRecordSnapshot> {
    let mut out: Vec<RegionalDnsRecordSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_dns_record_snapshot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_regional_dns_record_snapshot(item: &RegionalDnsRecordSnapshot) -> String {
    health_check_cycle::normalize_batched_health_check_budget_code(&item.code)
}
