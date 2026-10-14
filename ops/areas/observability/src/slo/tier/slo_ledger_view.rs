use super::*;

use acme_kit::text;

/// How screens and exports show a slo ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerView {
    pub code_text: String,
    pub name_text: String,
    pub priority: String,
    pub source: String,
    pub limit: String,
    pub batch_size: String,
    pub budget: String,
    pub category_text: String,
    pub weight: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_slo_ledger_view(item: &SloLedger) -> SloLedgerView {
    SloLedgerView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        source: text::truncate(&item.source, 64),
        limit: item.limit.to_string(),
        batch_size: item.batch_size.to_string(),
        budget: format!("{:.2}", item.budget),
        category_text: item.category.clone(),
        weight: format!("{:.2}", item.weight),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_ledger_views(items: &[SloLedger]) -> Vec<SloLedgerView> {
    let mut views: Vec<SloLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_ledger_view(item));
    }
    views
}

pub fn sum_slo_ledger_view(view: &SloLedgerView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_limit as f64) + (parsed_batch_size as f64))
}
