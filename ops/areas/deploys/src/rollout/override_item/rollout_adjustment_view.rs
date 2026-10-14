use super::*;

use acme_kit::text;

/// How screens and exports show a rollout adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAdjustmentView {
    pub code_text: String,
    pub name: String,
    pub timezone: String,
    pub cluster_text: String,
    pub visible_text: String,
    pub quota_units: String,
    pub duration_minutes: String,
    pub source: String,
    pub attempts: String,
    pub port: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_rollout_adjustment_view(item: &RolloutAdjustment) -> RolloutAdjustmentView {
    RolloutAdjustmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        timezone: item.timezone.clone(),
        cluster_text: item.cluster.clone(),
        visible_text: format!("{}", item.visible),
        quota_units: item.quota_units.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        source: text::truncate(&item.source, 80),
        attempts: item.attempts.to_string(),
        port: item.port.to_string(),
        cipher: text::truncate(&item.cipher, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_adjustment_views(items: &[RolloutAdjustment]) -> Vec<RolloutAdjustmentView> {
    let mut views: Vec<RolloutAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_adjustment_view(item));
    }
    views
}

pub fn sum_rollout_adjustment_view(view: &RolloutAdjustmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_duration_minutes as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_adjustment("code-858", "name-696");
        let view = to_rollout_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
