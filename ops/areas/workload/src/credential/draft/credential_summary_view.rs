use super::*;

use acme_kit::text;

/// How screens and exports show a credential summary.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummaryView {
    pub code: String,
    pub label: String,
    pub starts_at: String,
    pub sequence_text: String,
    pub ceiling_text: String,
    pub port_text: String,
    pub locale: String,
    pub reviewer: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_credential_summary_view(item: &CredentialSummary) -> CredentialSummaryView {
    CredentialSummaryView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 120),
        starts_at: item.starts_at.to_string(),
        sequence_text: item.sequence.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        port_text: item.port.to_string(),
        locale: text::truncate(&item.locale, 80),
        reviewer: item.reviewer.clone(),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_summary_views(items: &[CredentialSummary]) -> Vec<CredentialSummaryView> {
    let mut views: Vec<CredentialSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_summary_view(item));
    }
    views
}

pub fn sum_credential_summary_view(view: &CredentialSummaryView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_sequence as f64) + parsed_ceiling)
}
