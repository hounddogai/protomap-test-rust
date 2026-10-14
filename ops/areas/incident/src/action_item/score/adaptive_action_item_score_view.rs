use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive action item score.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemScoreView {
    pub code: String,
    pub label: String,
    pub cluster_text: String,
    pub attempts: String,
    pub retries: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_adaptive_action_item_score_view(item: &AdaptiveActionItemScore) -> AdaptiveActionItemScoreView {
    AdaptiveActionItemScoreView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        cluster_text: text::truncate(&item.cluster, 120),
        attempts: item.attempts.to_string(),
        retries: item.retries.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_action_item_score_views(items: &[AdaptiveActionItemScore]) -> Vec<AdaptiveActionItemScoreView> {
    let mut views: Vec<AdaptiveActionItemScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_action_item_score_view(item));
    }
    views
}

pub fn sum_adaptive_action_item_score_view(view: &AdaptiveActionItemScoreView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_retries as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_action_item_score("code-537", "label-624");
        let view = to_adaptive_action_item_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_action_item_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
