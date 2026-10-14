use super::*;

use acme_kit::text;

/// How screens and exports show a weighted allowlist checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAllowlistCheckpointView {
    pub code: String,
    pub label_text: String,
    pub score: String,
    pub channel_text: String,
    pub quota_units_text: String,
    pub segment: String,
    pub burst: String,
    pub description: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_weighted_allowlist_checkpoint_view(item: &WeightedAllowlistCheckpoint) -> WeightedAllowlistCheckpointView {
    WeightedAllowlistCheckpointView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        score: format!("{:.2}", item.score),
        channel_text: item.channel.clone(),
        quota_units_text: item.quota_units.to_string(),
        segment: text::truncate(&item.segment, 120),
        burst: item.burst.to_string(),
        description: item.description.clone(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_allowlist_checkpoint_views(
    items: &[WeightedAllowlistCheckpoint],
) -> Vec<WeightedAllowlistCheckpointView> {
    let mut views: Vec<WeightedAllowlistCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_allowlist_checkpoint_view(item));
    }
    views
}

pub fn sum_weighted_allowlist_checkpoint_view(view: &WeightedAllowlistCheckpointView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_score + (parsed_quota_units as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_allowlist_checkpoint("code-786", "label-89");
        let view = to_weighted_allowlist_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_allowlist_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
