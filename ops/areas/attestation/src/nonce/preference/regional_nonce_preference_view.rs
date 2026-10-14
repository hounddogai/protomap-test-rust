use super::*;

use acme_kit::text;

/// How screens and exports show a regional nonce preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNoncePreferenceView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub timezone: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_regional_nonce_preference_view(item: &RegionalNoncePreference) -> RegionalNoncePreferenceView {
    RegionalNoncePreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: text::truncate(&item.country, 80),
        timezone: item.timezone.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_nonce_preference_views(items: &[RegionalNoncePreference]) -> Vec<RegionalNoncePreferenceView> {
    let mut views: Vec<RegionalNoncePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_nonce_preference_view(item));
    }
    views
}

pub fn sum_regional_nonce_preference_view(view: &RegionalNoncePreferenceView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_nonce_preference("code-93", "title-286");
        let view = to_regional_nonce_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_nonce_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
