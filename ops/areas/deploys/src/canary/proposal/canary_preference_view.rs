use super::*;

use acme_kit::text;

/// How screens and exports show a canary preference.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreferenceView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub grace_days: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_canary_preference_view(item: &CanaryPreference) -> CanaryPreferenceView {
    CanaryPreferenceView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        budget: format!("{:.2}", item.budget),
        grace_days: item.grace_days.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_preference_views(items: &[CanaryPreference]) -> Vec<CanaryPreferenceView> {
    let mut views: Vec<CanaryPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_preference_view(item));
    }
    views
}

pub fn sum_canary_preference_view(view: &CanaryPreferenceView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_budget + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_preference("code-252", "title-169");
        let view = to_canary_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
