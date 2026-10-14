use super::*;

use acme_kit::text;

/// How screens and exports show a suspension score.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreView {
    pub code: String,
    pub label_text: String,
    pub description: String,
    pub discount_text: String,
    pub threshold: String,
    pub contact_email_text: String,
    pub scope_text: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_suspension_score_view(item: &SuspensionScore) -> SuspensionScoreView {
    SuspensionScoreView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        description: text::truncate(&item.description, 80),
        discount_text: format!("{:.2}", item.discount),
        threshold: format!("{:.2}", item.threshold),
        contact_email_text: item.contact_email.clone(),
        scope_text: item.scope.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_score_views(items: &[SuspensionScore]) -> Vec<SuspensionScoreView> {
    let mut views: Vec<SuspensionScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_score_view(item));
    }
    views
}

pub fn sum_suspension_score_view(view: &SuspensionScoreView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_discount + parsed_threshold + (parsed_duration_minutes as f64))
}
