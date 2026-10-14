use super::*;

use acme_kit::text;

/// How screens and exports show a federation preference.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationPreferenceView {
    pub code: String,
    pub title_text: String,
    pub cluster_text: String,
    pub phone: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_federation_preference_view(item: &FederationPreference) -> FederationPreferenceView {
    FederationPreferenceView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        cluster_text: item.cluster.clone(),
        phone: text::truncate(&item.phone, 80),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_preference_views(items: &[FederationPreference]) -> Vec<FederationPreferenceView> {
    let mut views: Vec<FederationPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_preference_view(item));
    }
    views
}

pub fn sum_federation_preference_view(view: &FederationPreferenceView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_preference("code-208", "title-948");
        let view = to_federation_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
