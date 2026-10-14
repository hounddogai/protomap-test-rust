use super::*;

/// How screens and exports show a fallback overage target.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetView {
    pub code: String,
    pub name_text: String,
    pub checksum: String,
    pub ceiling_text: String,
    pub region: String,
    pub category_text: String,
    pub grace_days: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_fallback_overage_target_view(item: &FallbackOverageTarget) -> FallbackOverageTargetView {
    FallbackOverageTargetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        checksum: item.checksum.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        region: item.region.clone(),
        category_text: item.category.clone(),
        grace_days: item.grace_days.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_overage_target_views(items: &[FallbackOverageTarget]) -> Vec<FallbackOverageTargetView> {
    let mut views: Vec<FallbackOverageTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_overage_target_view(item));
    }
    views
}

pub fn sum_fallback_overage_target_view(view: &FallbackOverageTargetView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ceiling + (parsed_grace_days as f64))
}
