use super::*;

use acme_kit::text;

/// How screens and exports show a health check agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreementView {
    pub code_text: String,
    pub title: String,
    pub cipher: String,
    pub reviewer: String,
    pub category_text: String,
    pub attempts: String,
    pub pinned: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_health_check_agreement_view(item: &HealthCheckAgreement) -> HealthCheckAgreementView {
    HealthCheckAgreementView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cipher: text::truncate(&item.cipher, 64),
        reviewer: item.reviewer.clone(),
        category_text: item.category.clone(),
        attempts: item.attempts.to_string(),
        pinned: format!("{}", item.pinned),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_agreement_views(items: &[HealthCheckAgreement]) -> Vec<HealthCheckAgreementView> {
    let mut views: Vec<HealthCheckAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_agreement_view(item));
    }
    views
}

pub fn sum_health_check_agreement_view(view: &HealthCheckAgreementView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_ceiling)
}
