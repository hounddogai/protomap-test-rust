use super::*;

use acme_kit::text;

/// How screens and exports show a strict rule preference.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRulePreferenceView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub cost: String,
    pub pinned: String,
    pub window_days: String,
    pub notes: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_strict_rule_preference_view(item: &StrictRulePreference) -> StrictRulePreferenceView {
    StrictRulePreferenceView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        scope: item.scope.clone(),
        cost: format!("{:.2}", item.cost),
        pinned: format!("{}", item.pinned),
        window_days: item.window_days.to_string(),
        notes: text::truncate(&item.notes, 40),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rule_preference_views(items: &[StrictRulePreference]) -> Vec<StrictRulePreferenceView> {
    let mut views: Vec<StrictRulePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rule_preference_view(item));
    }
    views
}

pub fn sum_strict_rule_preference_view(view: &StrictRulePreferenceView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_cost + (parsed_window_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_rule_preference("code-41", "label-806");
        let view = to_strict_rule_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_rule_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
