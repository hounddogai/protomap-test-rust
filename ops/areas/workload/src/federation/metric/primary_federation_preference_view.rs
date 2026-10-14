use super::*;

use acme_kit::text;

/// How screens and exports show a primary federation preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationPreferenceView {
    pub code_text: String,
    pub label: String,
    pub grace_days_text: String,
    pub unit: String,
    pub ratio: String,
    pub title_text: String,
    pub host: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_primary_federation_preference_view(item: &PrimaryFederationPreference) -> PrimaryFederationPreferenceView {
    PrimaryFederationPreferenceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        unit: text::truncate(&item.unit, 40),
        ratio: format!("{:.2}", item.ratio),
        title_text: item.title.clone(),
        host: item.host.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_federation_preference_views(
    items: &[PrimaryFederationPreference],
) -> Vec<PrimaryFederationPreferenceView> {
    let mut views: Vec<PrimaryFederationPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_federation_preference_view(item));
    }
    views
}

pub fn sum_primary_federation_preference_view(view: &PrimaryFederationPreferenceView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_federation_preference("code-333", "label-816");
        let view = to_primary_federation_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_federation_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
