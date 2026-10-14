use super::*;

use acme_kit::text;

/// How screens and exports show a slo agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementView {
    pub code_text: String,
    pub label: String,
    pub checksum_text: String,
    pub source_text: String,
    pub region: String,
    pub maximum: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_slo_agreement_view(item: &SloAgreement) -> SloAgreementView {
    SloAgreementView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        checksum_text: item.checksum.clone(),
        source_text: text::truncate(&item.source, 80),
        region: text::truncate(&item.region, 120),
        maximum: format!("{:.2}", item.maximum),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_agreement_views(items: &[SloAgreement]) -> Vec<SloAgreementView> {
    let mut views: Vec<SloAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_agreement_view(item));
    }
    views
}

pub fn sum_slo_agreement_view(view: &SloAgreementView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_agreement("code-50", "label-868");
        let view = to_slo_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
