use super::*;

use acme_kit::text;

/// How screens and exports show a weighted credential preference.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferenceView {
    pub code_text: String,
    pub name: String,
    pub discount: String,
    pub maximum: String,
    pub priority: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_weighted_credential_preference_view(item: &WeightedCredentialPreference) -> WeightedCredentialPreferenceView {
    WeightedCredentialPreferenceView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        discount: format!("{:.2}", item.discount),
        maximum: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_credential_preference_views(
    items: &[WeightedCredentialPreference],
) -> Vec<WeightedCredentialPreferenceView> {
    let mut views: Vec<WeightedCredentialPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_credential_preference_view(item));
    }
    views
}

pub fn sum_weighted_credential_preference_view(view: &WeightedCredentialPreferenceView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_discount + parsed_maximum + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_credential_preference("code-562", "name-795");
        let view = to_weighted_credential_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_credential_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
