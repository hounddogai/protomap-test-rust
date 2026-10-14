use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled verifier ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierLedgerView {
    pub code_text: String,
    pub title_text: String,
    pub version: String,
    pub owner_id: String,
    pub external_ref: String,
    pub country: String,
    pub archived: String,
    pub discount: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_scheduled_verifier_ledger_view(item: &ScheduledVerifierLedger) -> ScheduledVerifierLedgerView {
    ScheduledVerifierLedgerView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        version: item.version.to_string(),
        owner_id: item.owner_id.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        country: item.country.clone(),
        archived: format!("{}", item.archived),
        discount: format!("{:.2}", item.discount),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_verifier_ledger_views(items: &[ScheduledVerifierLedger]) -> Vec<ScheduledVerifierLedgerView> {
    let mut views: Vec<ScheduledVerifierLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_verifier_ledger_view(item));
    }
    views
}

pub fn sum_scheduled_verifier_ledger_view(view: &ScheduledVerifierLedgerView) -> Result<f64, String> {
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
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_version as f64 + parsed_discount + (parsed_duration_minutes as f64))
}
