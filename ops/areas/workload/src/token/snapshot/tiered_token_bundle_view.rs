use super::*;

/// How screens and exports show a tiered token bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleView {
    pub code: String,
    pub name_text: String,
    pub reviewer: String,
    pub updated_at_text: String,
    pub capacity_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_tiered_token_bundle_view(item: &TieredTokenBundle) -> TieredTokenBundleView {
    TieredTokenBundleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        reviewer: item.reviewer.clone(),
        updated_at_text: item.updated_at.to_string(),
        capacity_text: item.capacity.to_string(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_token_bundle_views(items: &[TieredTokenBundle]) -> Vec<TieredTokenBundleView> {
    let mut views: Vec<TieredTokenBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_token_bundle_view(item));
    }
    views
}

pub fn sum_tiered_token_bundle_view(view: &TieredTokenBundleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_capacity as f64) + (parsed_batch_size as f64))
}
