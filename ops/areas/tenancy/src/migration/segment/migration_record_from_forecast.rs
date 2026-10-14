use super::*;

use std::collections::BTreeMap;

use crate::tenant::score as tenant_score;

pub fn summarize_tenant_forecasts_for_record(items: &[tenant_score::TenantForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_migration_records_from_forecast(items: &[tenant_score::TenantForecast]) -> Vec<MigrationRecord> {
    let mut out: Vec<MigrationRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_record(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_migration_record(item: &MigrationRecord) -> String {
    tenant_score::tenant_score_bucket(item.projected_exposure())
}
