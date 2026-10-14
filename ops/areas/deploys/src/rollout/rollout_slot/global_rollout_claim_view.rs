use super::*;

/// How screens and exports show a global rollout claim.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub duration_minutes_text: String,
    pub window_days: String,
    pub summary: String,
    pub discount: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_global_rollout_claim_view(item: &GlobalRolloutClaim) -> GlobalRolloutClaimView {
    GlobalRolloutClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        window_days: item.window_days.to_string(),
        summary: item.summary.clone(),
        discount: format!("{:.2}", item.discount),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_rollout_claim_views(items: &[GlobalRolloutClaim]) -> Vec<GlobalRolloutClaimView> {
    let mut views: Vec<GlobalRolloutClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_rollout_claim_view(item));
    }
    views
}

pub fn sum_global_rollout_claim_view(view: &GlobalRolloutClaimView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_window_days as f64) + parsed_discount)
}
