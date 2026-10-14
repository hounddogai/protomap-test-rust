use super::*;

use acme_kit::text;

/// How screens and exports show a probe ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeLedgerView {
    pub code: String,
    pub title: String,
    pub tenant_id_text: String,
    pub ceiling: String,
    pub starts_at: String,
    pub fingerprint: String,
    pub issuer: String,
    pub retries: String,
    pub rotation_days: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_probe_ledger_view(item: &ProbeLedger) -> ProbeLedgerView {
    ProbeLedgerView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        tenant_id_text: text::truncate(&item.tenant_id, 80),
        ceiling: format!("{:.2}", item.ceiling),
        starts_at: item.starts_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        issuer: item.issuer.clone(),
        retries: item.retries.to_string(),
        rotation_days: item.rotation_days.to_string(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_ledger_views(items: &[ProbeLedger]) -> Vec<ProbeLedgerView> {
    let mut views: Vec<ProbeLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_ledger_view(item));
    }
    views
}

pub fn sum_probe_ledger_view(view: &ProbeLedgerView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ceiling + (parsed_starts_at as f64) + (parsed_retries as f64))
}
