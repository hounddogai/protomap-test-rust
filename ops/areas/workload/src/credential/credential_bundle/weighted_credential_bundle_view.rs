use super::*;

use acme_kit::text;

/// How screens and exports show a weighted credential bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleView {
    pub code: String,
    pub title_text: String,
    pub country: String,
    pub rate: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_weighted_credential_bundle_view(item: &WeightedCredentialBundle) -> WeightedCredentialBundleView {
    WeightedCredentialBundleView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        country: text::truncate(&item.country, 40),
        rate: format!("{:.2}", item.rate),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_credential_bundle_views(items: &[WeightedCredentialBundle]) -> Vec<WeightedCredentialBundleView> {
    let mut views: Vec<WeightedCredentialBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_credential_bundle_view(item));
    }
    views
}

pub fn sum_weighted_credential_bundle_view(view: &WeightedCredentialBundleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_credential_bundle("code-656", "title-115");
        let view = to_weighted_credential_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_credential_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
