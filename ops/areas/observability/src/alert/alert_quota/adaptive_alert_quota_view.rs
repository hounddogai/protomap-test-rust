use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive alert quota.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertQuotaView {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub host: String,
    pub score: String,
    pub locale: String,
    pub title_text: String,
    pub margin: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_adaptive_alert_quota_view(item: &AdaptiveAlertQuota) -> AdaptiveAlertQuotaView {
    AdaptiveAlertQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        scope: item.scope.clone(),
        host: item.host.clone(),
        score: format!("{:.2}", item.score),
        locale: item.locale.clone(),
        title_text: text::truncate(&item.title, 80),
        margin: format!("{:.2}", item.margin),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_alert_quota_views(items: &[AdaptiveAlertQuota]) -> Vec<AdaptiveAlertQuotaView> {
    let mut views: Vec<AdaptiveAlertQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_alert_quota_view(item));
    }
    views
}

pub fn sum_adaptive_alert_quota_view(view: &AdaptiveAlertQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_score + parsed_margin + parsed_rate)
}
