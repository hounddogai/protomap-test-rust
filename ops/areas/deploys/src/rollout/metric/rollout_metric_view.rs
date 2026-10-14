use super::*;

use acme_kit::text;

/// How screens and exports show a rollout metric.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetricView {
    pub code: String,
    pub title: String,
    pub segment_text: String,
    pub currency: String,
    pub notes_text: String,
    pub key_id: String,
    pub ttl_seconds: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_rollout_metric_view(item: &RolloutMetric) -> RolloutMetricView {
    RolloutMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        segment_text: item.segment.clone(),
        currency: item.currency.clone(),
        notes_text: item.notes.clone(),
        key_id: item.key_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        owner_id: text::truncate(&item.owner_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_metric_views(items: &[RolloutMetric]) -> Vec<RolloutMetricView> {
    let mut views: Vec<RolloutMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_metric_view(item));
    }
    views
}

pub fn sum_rollout_metric_view(view: &RolloutMetricView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_metric("code-520", "title-98");
        let view = to_rollout_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
