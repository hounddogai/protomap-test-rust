use super::*;

use std::collections::BTreeMap;

use crate::tenant::metric as tenant_metric;

pub fn summarize_tenant_metrics_for_quota(items: &[tenant_metric::TenantMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_adaptive_suspension_quotas_from_metric(
    items: &[tenant_metric::TenantMetric],
) -> Vec<AdaptiveSuspensionQuota> {
    let mut out: Vec<AdaptiveSuspensionQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_suspension_quota(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_adaptive_suspension_quota(item: &AdaptiveSuspensionQuota) -> String {
    tenant_metric::tenant_metric_bucket(item.weighted_load())
}
