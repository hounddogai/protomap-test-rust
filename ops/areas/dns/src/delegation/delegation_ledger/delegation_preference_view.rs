use super::*;

use acme_kit::text;

/// How screens and exports show a delegation preference.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPreferenceView {
    pub code: String,
    pub title: String,
    pub discount_text: String,
    pub scope_text: String,
    pub threshold_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_delegation_preference_view(item: &DelegationPreference) -> DelegationPreferenceView {
    DelegationPreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        scope_text: text::truncate(&item.scope, 40),
        threshold_text: format!("{:.2}", item.threshold),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_preference_views(items: &[DelegationPreference]) -> Vec<DelegationPreferenceView> {
    let mut views: Vec<DelegationPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_preference_view(item));
    }
    views
}

pub fn sum_delegation_preference_view(view: &DelegationPreferenceView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_discount + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_preference("code-806", "title-986");
        let view = to_delegation_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
