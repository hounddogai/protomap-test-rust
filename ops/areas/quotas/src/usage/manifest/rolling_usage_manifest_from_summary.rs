use super::*;

use std::collections::BTreeMap;

use crate::overage::summary as overage_summary;

pub fn summarize_capped_overage_summaries_for_manifest(
    items: &[overage_summary::CappedOverageSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_rolling_usage_manifests_from_summary(
    items: &[overage_summary::CappedOverageSummary],
) -> Vec<RollingUsageManifest> {
    let mut out: Vec<RollingUsageManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_usage_manifest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_rolling_usage_manifest(item: &RollingUsageManifest) -> String {
    overage_summary::normalize_capped_overage_summary_code(&item.code)
}
