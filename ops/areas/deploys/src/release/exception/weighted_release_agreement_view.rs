use super::*;

use acme_kit::text;

/// How screens and exports show a weighted release agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReleaseAgreementView {
    pub code: String,
    pub label_text: String,
    pub host: String,
    pub ratio: String,
    pub principal: String,
    pub ends_at: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_weighted_release_agreement_view(item: &WeightedReleaseAgreement) -> WeightedReleaseAgreementView {
    WeightedReleaseAgreementView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        host: item.host.clone(),
        ratio: format!("{:.2}", item.ratio),
        principal: text::truncate(&item.principal, 80),
        ends_at: item.ends_at.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_release_agreement_views(items: &[WeightedReleaseAgreement]) -> Vec<WeightedReleaseAgreementView> {
    let mut views: Vec<WeightedReleaseAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_release_agreement_view(item));
    }
    views
}

pub fn sum_weighted_release_agreement_view(view: &WeightedReleaseAgreementView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ratio + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_release_agreement("code-255", "label-174");
        let view = to_weighted_release_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_release_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
