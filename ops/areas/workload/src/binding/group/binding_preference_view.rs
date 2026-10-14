use super::*;

use acme_kit::text;

/// How screens and exports show a binding preference.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreferenceView {
    pub code: String,
    pub name: String,
    pub owner_id_text: String,
    pub phone: String,
    pub created_at: String,
    pub score_text: String,
    pub rotation_days_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_binding_preference_view(item: &BindingPreference) -> BindingPreferenceView {
    BindingPreferenceView {
        code: item.code.clone(),
        name: item.name.clone(),
        owner_id_text: text::truncate(&item.owner_id, 80),
        phone: text::truncate(&item.phone, 80),
        created_at: item.created_at.to_string(),
        score_text: format!("{:.2}", item.score),
        rotation_days_text: item.rotation_days.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_preference_views(items: &[BindingPreference]) -> Vec<BindingPreferenceView> {
    let mut views: Vec<BindingPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_preference_view(item));
    }
    views
}

pub fn sum_binding_preference_view(view: &BindingPreferenceView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_score + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_preference("code-363", "name-469");
        let view = to_binding_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
