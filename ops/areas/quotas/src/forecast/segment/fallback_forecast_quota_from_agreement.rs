use super::*;

use std::collections::BTreeMap;

use crate::overage::quota as overage_quota;

pub fn summarize_tiered_overage_agreements_for_quota(
    items: &[overage_quota::TieredOverageAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_fallback_forecast_quotas_from_agreement(
    items: &[overage_quota::TieredOverageAgreement],
) -> Vec<FallbackForecastQuota> {
    let mut out: Vec<FallbackForecastQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_forecast_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_forecast_quota(item: &FallbackForecastQuota) -> String {
    overage_quota::capped_overage_quota_bucket(item.net_value())
}
