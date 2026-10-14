use super::*;

use acme_kit::text;

/// How screens and exports show a rolling peering override.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringOverrideView {
    pub code: String,
    pub name_text: String,
    pub external_ref: String,
    pub attempts_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_rolling_peering_override_view(item: &RollingPeeringOverride) -> RollingPeeringOverrideView {
    RollingPeeringOverrideView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        external_ref: item.external_ref.clone(),
        attempts_text: item.attempts.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_peering_override_views(items: &[RollingPeeringOverride]) -> Vec<RollingPeeringOverrideView> {
    let mut views: Vec<RollingPeeringOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_peering_override_view(item));
    }
    views
}

pub fn sum_rolling_peering_override_view(view: &RollingPeeringOverrideView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_peering_override("code-180", "name-876");
        let view = to_rolling_peering_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_peering_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
