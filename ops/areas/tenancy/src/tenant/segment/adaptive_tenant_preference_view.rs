use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive tenant preference.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreferenceView {
    pub code: String,
    pub name_text: String,
    pub maximum: String,
    pub fingerprint: String,
    pub rate: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_adaptive_tenant_preference_view(item: &AdaptiveTenantPreference) -> AdaptiveTenantPreferenceView {
    AdaptiveTenantPreferenceView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        maximum: format!("{:.2}", item.maximum),
        fingerprint: item.fingerprint.clone(),
        rate: format!("{:.2}", item.rate),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_tenant_preference_views(items: &[AdaptiveTenantPreference]) -> Vec<AdaptiveTenantPreferenceView> {
    let mut views: Vec<AdaptiveTenantPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_tenant_preference_view(item));
    }
    views
}

pub fn sum_adaptive_tenant_preference_view(view: &AdaptiveTenantPreferenceView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_maximum + parsed_rate + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_tenant_preference("code-279", "name-724");
        let view = to_adaptive_tenant_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_tenant_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
