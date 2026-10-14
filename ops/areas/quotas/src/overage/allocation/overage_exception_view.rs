use super::*;

use acme_kit::text;

/// How screens and exports show a overage exception.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionView {
    pub code_text: String,
    pub name_text: String,
    pub maximum: String,
    pub reviewer: String,
    pub key_id_text: String,
    pub rotation_days: String,
    pub category: String,
    pub country: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_overage_exception_view(item: &OverageException) -> OverageExceptionView {
    OverageExceptionView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        maximum: format!("{:.2}", item.maximum),
        reviewer: text::truncate(&item.reviewer, 120),
        key_id_text: item.key_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        category: text::truncate(&item.category, 40),
        country: text::truncate(&item.country, 120),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_exception_views(items: &[OverageException]) -> Vec<OverageExceptionView> {
    let mut views: Vec<OverageExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_exception_view(item));
    }
    views
}

pub fn sum_overage_exception_view(view: &OverageExceptionView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_maximum + (parsed_rotation_days as f64) + (parsed_created_at as f64))
}
