use super::*;

use acme_kit::text;

/// How screens and exports show a health check checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointView {
    pub code: String,
    pub label_text: String,
    pub scope_text: String,
    pub rate: String,
    pub attempts: String,
    pub ttl_seconds: String,
    pub category_text: String,
    pub minimum: String,
    pub source: String,
    pub key_id: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_health_check_checkpoint_view(item: &HealthCheckCheckpoint) -> HealthCheckCheckpointView {
    HealthCheckCheckpointView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        scope_text: text::truncate(&item.scope, 64),
        rate: format!("{:.2}", item.rate),
        attempts: item.attempts.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        category_text: item.category.clone(),
        minimum: format!("{:.2}", item.minimum),
        source: item.source.clone(),
        key_id: item.key_id.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_checkpoint_views(items: &[HealthCheckCheckpoint]) -> Vec<HealthCheckCheckpointView> {
    let mut views: Vec<HealthCheckCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_checkpoint_view(item));
    }
    views
}

pub fn sum_health_check_checkpoint_view(view: &HealthCheckCheckpointView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_rate + (parsed_attempts as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_checkpoint("code-807", "label-838");
        let view = to_health_check_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
