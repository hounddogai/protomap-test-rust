use super::*;

use acme_kit::text;

/// How screens and exports show a tiered measurement agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementView {
    pub code_text: String,
    pub name: String,
    pub title_text: String,
    pub pinned: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_tiered_measurement_agreement_view(item: &TieredMeasurementAgreement) -> TieredMeasurementAgreementView {
    TieredMeasurementAgreementView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        title_text: text::truncate(&item.title, 40),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_measurement_agreement_views(
    items: &[TieredMeasurementAgreement],
) -> Vec<TieredMeasurementAgreementView> {
    let mut views: Vec<TieredMeasurementAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_measurement_agreement_view(item));
    }
    views
}

pub fn sum_tiered_measurement_agreement_view(view: &TieredMeasurementAgreementView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
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
        let item = new_tiered_measurement_agreement("code-918", "name-533");
        let view = to_tiered_measurement_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_measurement_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
