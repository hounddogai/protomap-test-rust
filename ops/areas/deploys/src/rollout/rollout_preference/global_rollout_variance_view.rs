use super::*;

use acme_kit::text;

/// How screens and exports show a global rollout variance.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceView {
    pub code: String,
    pub label: String,
    pub ttl_seconds_text: String,
    pub phone_text: String,
    pub visible: String,
    pub rotation_days: String,
    pub reviewer_text: String,
    pub checksum: String,
    pub quota_units: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_global_rollout_variance_view(item: &GlobalRolloutVariance) -> GlobalRolloutVarianceView {
    GlobalRolloutVarianceView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        phone_text: text::truncate(&item.phone, 64),
        visible: format!("{}", item.visible),
        rotation_days: item.rotation_days.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        checksum: item.checksum.clone(),
        quota_units: item.quota_units.to_string(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_rollout_variance_views(items: &[GlobalRolloutVariance]) -> Vec<GlobalRolloutVarianceView> {
    let mut views: Vec<GlobalRolloutVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_rollout_variance_view(item));
    }
    views
}

pub fn sum_global_rollout_variance_view(view: &GlobalRolloutVarianceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_rotation_days as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_rollout_variance("code-445", "label-525");
        let view = to_global_rollout_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_rollout_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
