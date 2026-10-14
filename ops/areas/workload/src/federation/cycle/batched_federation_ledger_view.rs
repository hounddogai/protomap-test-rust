use super::*;

use acme_kit::text;

/// How screens and exports show a batched federation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub contact_email: String,
    pub channel_text: String,
    pub country: String,
    pub maximum: String,
    pub rate_text: String,
    pub locale_text: String,
    pub active_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_batched_federation_ledger_view(item: &BatchedFederationLedger) -> BatchedFederationLedgerView {
    BatchedFederationLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 40),
        channel_text: item.channel.clone(),
        country: item.country.clone(),
        maximum: format!("{:.2}", item.maximum),
        rate_text: format!("{:.2}", item.rate),
        locale_text: item.locale.clone(),
        active_text: format!("{}", item.active),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_federation_ledger_views(items: &[BatchedFederationLedger]) -> Vec<BatchedFederationLedgerView> {
    let mut views: Vec<BatchedFederationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_federation_ledger_view(item));
    }
    views
}

pub fn sum_batched_federation_ledger_view(view: &BatchedFederationLedgerView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_maximum + parsed_rate)
}
