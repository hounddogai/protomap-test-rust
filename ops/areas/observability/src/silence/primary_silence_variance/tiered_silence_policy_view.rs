use super::*;

/// How screens and exports show a tiered silence policy.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyView {
    pub code: String,
    pub name: String,
    pub ratio: String,
    pub tenant_id_text: String,
    pub created_at: String,
    pub discount: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_tiered_silence_policy_view(item: &TieredSilencePolicy) -> TieredSilencePolicyView {
    TieredSilencePolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        ratio: format!("{:.2}", item.ratio),
        tenant_id_text: item.tenant_id.clone(),
        created_at: item.created_at.to_string(),
        discount: format!("{:.2}", item.discount),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_silence_policy_views(items: &[TieredSilencePolicy]) -> Vec<TieredSilencePolicyView> {
    let mut views: Vec<TieredSilencePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_silence_policy_view(item));
    }
    views
}

pub fn sum_tiered_silence_policy_view(view: &TieredSilencePolicyView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ratio + (parsed_created_at as f64) + parsed_discount)
}
