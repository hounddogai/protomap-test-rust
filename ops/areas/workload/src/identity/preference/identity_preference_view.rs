use super::*;

use acme_kit::text;

/// How screens and exports show a identity preference.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPreferenceView {
    pub code: String,
    pub name_text: String,
    pub priority_text: String,
    pub notes: String,
    pub limit: String,
    pub channel: String,
    pub budget_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_identity_preference_view(item: &IdentityPreference) -> IdentityPreferenceView {
    IdentityPreferenceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        priority_text: item.priority.to_string(),
        notes: item.notes.clone(),
        limit: item.limit.to_string(),
        channel: text::truncate(&item.channel, 120),
        budget_text: format!("{:.2}", item.budget),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_preference_views(items: &[IdentityPreference]) -> Vec<IdentityPreferenceView> {
    let mut views: Vec<IdentityPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_preference_view(item));
    }
    views
}

pub fn sum_identity_preference_view(view: &IdentityPreferenceView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_limit as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_preference("code-786", "name-645");
        let view = to_identity_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
