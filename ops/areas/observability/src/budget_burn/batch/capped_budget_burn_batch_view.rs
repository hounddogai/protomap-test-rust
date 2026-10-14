use super::*;

use acme_kit::text;

/// How screens and exports show a capped budget burn batch.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnBatchView {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub retries: String,
    pub version: String,
    pub discount: String,
    pub checksum: String,
    pub created_at: String,
    pub category: String,
    pub quantity: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_batch_view(item: &CappedBudgetBurnBatch) -> CappedBudgetBurnBatchView {
    CappedBudgetBurnBatchView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        fingerprint: item.fingerprint.clone(),
        retries: item.retries.to_string(),
        version: item.version.to_string(),
        discount: format!("{:.2}", item.discount),
        checksum: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        category: item.category.clone(),
        quantity: item.quantity.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_batch_views(items: &[CappedBudgetBurnBatch]) -> Vec<CappedBudgetBurnBatchView> {
    let mut views: Vec<CappedBudgetBurnBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_batch_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_batch_view(view: &CappedBudgetBurnBatchView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_version as f64) + parsed_discount)
}
