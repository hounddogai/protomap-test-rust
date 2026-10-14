use super::*;

use acme_kit::text;

/// How screens and exports show a limit agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementView {
    pub code: String,
    pub title_text: String,
    pub cost_text: String,
    pub attempts: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_limit_agreement_view(item: &LimitAgreement) -> LimitAgreementView {
    LimitAgreementView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        cost_text: format!("{:.2}", item.cost),
        attempts: item.attempts.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_agreement_views(items: &[LimitAgreement]) -> Vec<LimitAgreementView> {
    let mut views: Vec<LimitAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_agreement_view(item));
    }
    views
}

pub fn sum_limit_agreement_view(view: &LimitAgreementView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64) + parsed_ratio)
}
