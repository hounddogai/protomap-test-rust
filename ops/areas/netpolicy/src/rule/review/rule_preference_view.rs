use super::*;

use acme_kit::text;

/// How screens and exports show a rule preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RulePreferenceView {
    pub code: String,
    pub title_text: String,
    pub owner_id: String,
    pub region: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_rule_preference_view(item: &RulePreference) -> RulePreferenceView {
    RulePreferenceView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        owner_id: item.owner_id.clone(),
        region: item.region.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_preference_views(items: &[RulePreference]) -> Vec<RulePreferenceView> {
    let mut views: Vec<RulePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_preference_view(item));
    }
    views
}

pub fn sum_rule_preference_view(view: &RulePreferenceView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_preference("code-81", "title-813");
        let view = to_rule_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
