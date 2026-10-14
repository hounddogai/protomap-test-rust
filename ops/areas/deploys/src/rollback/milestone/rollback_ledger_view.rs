use super::*;

use acme_kit::text;

/// How screens and exports show a rollback ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackLedgerView {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub principal_text: String,
    pub summary: String,
    pub namespace: String,
    pub ratio: String,
    pub label: String,
    pub host: String,
    pub version_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_rollback_ledger_view(item: &RollbackLedger) -> RollbackLedgerView {
    RollbackLedgerView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        scope: item.scope.clone(),
        principal_text: item.principal.clone(),
        summary: item.summary.clone(),
        namespace: item.namespace.clone(),
        ratio: format!("{:.2}", item.ratio),
        label: item.label.clone(),
        host: text::truncate(&item.host, 64),
        version_text: item.version.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_ledger_views(items: &[RollbackLedger]) -> Vec<RollbackLedgerView> {
    let mut views: Vec<RollbackLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_ledger_view(item));
    }
    views
}

pub fn sum_rollback_ledger_view(view: &RollbackLedgerView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ratio + (parsed_version as f64))
}
