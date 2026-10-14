use super::*;

/// How screens and exports show a adaptive release tier.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierView {
    pub code: String,
    pub label: String,
    pub archived: String,
    pub starts_at_text: String,
    pub duration_minutes: String,
    pub namespace: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_adaptive_release_tier_view(item: &AdaptiveReleaseTier) -> AdaptiveReleaseTierView {
    AdaptiveReleaseTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        archived: format!("{}", item.archived),
        starts_at_text: item.starts_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        namespace: item.namespace.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_release_tier_views(items: &[AdaptiveReleaseTier]) -> Vec<AdaptiveReleaseTierView> {
    let mut views: Vec<AdaptiveReleaseTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_release_tier_view(item));
    }
    views
}

pub fn sum_adaptive_release_tier_view(view: &AdaptiveReleaseTierView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_duration_minutes as f64) + parsed_score)
}
