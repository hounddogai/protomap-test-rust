use super::*;

use acme_kit::text;

/// How screens and exports show a rolling token digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestView {
    pub code: String,
    pub title: String,
    pub version: String,
    pub discount: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_rolling_token_digest_view(item: &RollingTokenDigest) -> RollingTokenDigestView {
    RollingTokenDigestView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        version: item.version.to_string(),
        discount: format!("{:.2}", item.discount),
        unit: text::truncate(&item.unit, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_token_digest_views(items: &[RollingTokenDigest]) -> Vec<RollingTokenDigestView> {
    let mut views: Vec<RollingTokenDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_token_digest_view(item));
    }
    views
}

pub fn sum_rolling_token_digest_view(view: &RollingTokenDigestView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_version as f64 + parsed_discount)
}
