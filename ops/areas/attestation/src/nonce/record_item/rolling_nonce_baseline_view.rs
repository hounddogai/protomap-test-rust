use super::*;

/// How screens and exports show a rolling nonce baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineView {
    pub code: String,
    pub title_text: String,
    pub checksum: String,
    pub discount: String,
    pub key_id_text: String,
    pub created_at_text: String,
    pub channel: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_rolling_nonce_baseline_view(item: &RollingNonceBaseline) -> RollingNonceBaselineView {
    RollingNonceBaselineView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        checksum: item.checksum.clone(),
        discount: format!("{:.2}", item.discount),
        key_id_text: item.key_id.clone(),
        created_at_text: item.created_at.to_string(),
        channel: item.channel.clone(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_nonce_baseline_views(items: &[RollingNonceBaseline]) -> Vec<RollingNonceBaselineView> {
    let mut views: Vec<RollingNonceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_nonce_baseline_view(item));
    }
    views
}

pub fn sum_rolling_nonce_baseline_view(view: &RollingNonceBaselineView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_discount + (parsed_created_at as f64))
}
