use super::*;

use acme_kit::text;

/// How screens and exports show a fallback redaction template.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionTemplateView {
    pub code: String,
    pub title_text: String,
    pub category_text: String,
    pub unit: String,
    pub approved_text: String,
    pub limit: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_fallback_redaction_template_view(item: &FallbackRedactionTemplate) -> FallbackRedactionTemplateView {
    FallbackRedactionTemplateView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        category_text: item.category.clone(),
        unit: item.unit.clone(),
        approved_text: format!("{}", item.approved),
        limit: item.limit.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_redaction_template_views(items: &[FallbackRedactionTemplate]) -> Vec<FallbackRedactionTemplateView> {
    let mut views: Vec<FallbackRedactionTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_redaction_template_view(item));
    }
    views
}

pub fn sum_fallback_redaction_template_view(view: &FallbackRedactionTemplateView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_limit as f64 + parsed_discount)
}
