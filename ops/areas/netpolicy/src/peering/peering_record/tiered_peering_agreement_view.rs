use super::*;

use acme_kit::text;

/// How screens and exports show a tiered peering agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAgreementView {
    pub code: String,
    pub title: String,
    pub audience_text: String,
    pub threshold_text: String,
    pub label: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_tiered_peering_agreement_view(item: &TieredPeeringAgreement) -> TieredPeeringAgreementView {
    TieredPeeringAgreementView {
        code: item.code.clone(),
        title: item.title.clone(),
        audience_text: text::truncate(&item.audience, 120),
        threshold_text: format!("{:.2}", item.threshold),
        label: item.label.clone(),
        external_ref: text::truncate(&item.external_ref, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_peering_agreement_views(items: &[TieredPeeringAgreement]) -> Vec<TieredPeeringAgreementView> {
    let mut views: Vec<TieredPeeringAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_peering_agreement_view(item));
    }
    views
}

pub fn sum_tiered_peering_agreement_view(view: &TieredPeeringAgreementView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_peering_agreement("code-496", "title-873");
        let view = to_tiered_peering_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_peering_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
