use super::*;

use acme_kit::text;

/// How screens and exports show a grant preference.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreferenceView {
    pub code_text: String,
    pub name: String,
    pub reviewer: String,
    pub description: String,
    pub approved: String,
    pub notes: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_grant_preference_view(item: &GrantPreference) -> GrantPreferenceView {
    GrantPreferenceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        reviewer: item.reviewer.clone(),
        description: text::truncate(&item.description, 64),
        approved: format!("{}", item.approved),
        notes: item.notes.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_preference_views(items: &[GrantPreference]) -> Vec<GrantPreferenceView> {
    let mut views: Vec<GrantPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_preference_view(item));
    }
    views
}

pub fn sum_grant_preference_view(view: &GrantPreferenceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_preference("code-426", "name-730");
        let view = to_grant_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
