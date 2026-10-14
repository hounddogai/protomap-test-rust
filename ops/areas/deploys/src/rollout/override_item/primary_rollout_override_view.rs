use super::*;

use acme_kit::text;

/// How screens and exports show a primary rollout override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideView {
    pub code: String,
    pub label: String,
    pub threshold: String,
    pub discount_text: String,
    pub duration_minutes: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_primary_rollout_override_view(item: &PrimaryRolloutOverride) -> PrimaryRolloutOverrideView {
    PrimaryRolloutOverrideView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 80),
        threshold: format!("{:.2}", item.threshold),
        discount_text: format!("{:.2}", item.discount),
        duration_minutes: item.duration_minutes.to_string(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollout_override_views(items: &[PrimaryRolloutOverride]) -> Vec<PrimaryRolloutOverrideView> {
    let mut views: Vec<PrimaryRolloutOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollout_override_view(item));
    }
    views
}

pub fn sum_primary_rollout_override_view(view: &PrimaryRolloutOverrideView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_threshold + parsed_discount + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rollout_override("code-620", "label-198");
        let view = to_primary_rollout_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rollout_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
