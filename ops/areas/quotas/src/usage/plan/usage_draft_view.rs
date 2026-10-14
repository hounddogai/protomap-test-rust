use super::*;

/// How screens and exports show a usage draft.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDraftView {
    pub code: String,
    pub label_text: String,
    pub limit: String,
    pub budget: String,
    pub audience: String,
    pub external_ref: String,
    pub name_text: String,
    pub country: String,
    pub ratio_text: String,
    pub region: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_usage_draft_view(item: &UsageDraft) -> UsageDraftView {
    UsageDraftView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        limit: item.limit.to_string(),
        budget: format!("{:.2}", item.budget),
        audience: item.audience.clone(),
        external_ref: item.external_ref.clone(),
        name_text: item.name.clone(),
        country: item.country.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        region: item.region.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_draft_views(items: &[UsageDraft]) -> Vec<UsageDraftView> {
    let mut views: Vec<UsageDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_draft_view(item));
    }
    views
}

pub fn sum_usage_draft_view(view: &UsageDraftView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_limit as f64 + parsed_budget + parsed_ratio)
}
