use super::*;

use acme_kit::text;

/// How screens and exports show a weighted runbook agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookAgreementView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub category_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_weighted_runbook_agreement_view(item: &WeightedRunbookAgreement) -> WeightedRunbookAgreementView {
    WeightedRunbookAgreementView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        maximum: format!("{:.2}", item.maximum),
        category_text: text::truncate(&item.category, 80),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_runbook_agreement_views(items: &[WeightedRunbookAgreement]) -> Vec<WeightedRunbookAgreementView> {
    let mut views: Vec<WeightedRunbookAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_runbook_agreement_view(item));
    }
    views
}

pub fn sum_weighted_runbook_agreement_view(view: &WeightedRunbookAgreementView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_runbook_agreement("code-2", "label-748");
        let view = to_weighted_runbook_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_runbook_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
