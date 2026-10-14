use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive canary tier.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryTierView {
    pub code: String,
    pub label: String,
    pub attempts_text: String,
    pub revision_text: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_adaptive_canary_tier_view(item: &AdaptiveCanaryTier) -> AdaptiveCanaryTierView {
    AdaptiveCanaryTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        attempts_text: item.attempts.to_string(),
        revision_text: item.revision.to_string(),
        key_id_text: text::truncate(&item.key_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_canary_tier_views(items: &[AdaptiveCanaryTier]) -> Vec<AdaptiveCanaryTierView> {
    let mut views: Vec<AdaptiveCanaryTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_canary_tier_view(item));
    }
    views
}

pub fn sum_adaptive_canary_tier_view(view: &AdaptiveCanaryTierView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_revision as f64))
}
