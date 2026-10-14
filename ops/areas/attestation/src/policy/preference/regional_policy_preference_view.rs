use super::*;

use acme_kit::text;

/// How screens and exports show a regional policy preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceView {
    pub code: String,
    pub label: String,
    pub ttl_seconds_text: String,
    pub sequence: String,
    pub starts_at: String,
    pub window_days: String,
    pub reviewer: String,
    pub threshold: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_regional_policy_preference_view(item: &RegionalPolicyPreference) -> RegionalPolicyPreferenceView {
    RegionalPolicyPreferenceView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        sequence: item.sequence.to_string(),
        starts_at: item.starts_at.to_string(),
        window_days: item.window_days.to_string(),
        reviewer: text::truncate(&item.reviewer, 120),
        threshold: format!("{:.2}", item.threshold),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_policy_preference_views(items: &[RegionalPolicyPreference]) -> Vec<RegionalPolicyPreferenceView> {
    let mut views: Vec<RegionalPolicyPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_policy_preference_view(item));
    }
    views
}

pub fn sum_regional_policy_preference_view(view: &RegionalPolicyPreferenceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_sequence as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_policy_preference("code-322", "label-351");
        let view = to_regional_policy_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_policy_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
