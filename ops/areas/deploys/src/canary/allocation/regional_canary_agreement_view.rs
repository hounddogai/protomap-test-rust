use super::*;

use acme_kit::text;

/// How screens and exports show a regional canary agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreementView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub namespace: String,
    pub currency_text: String,
    pub source: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_regional_canary_agreement_view(item: &RegionalCanaryAgreement) -> RegionalCanaryAgreementView {
    RegionalCanaryAgreementView {
        code: item.code.clone(),
        name: item.name.clone(),
        scope: text::truncate(&item.scope, 40),
        namespace: item.namespace.clone(),
        currency_text: item.currency.clone(),
        source: text::truncate(&item.source, 64),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_canary_agreement_views(items: &[RegionalCanaryAgreement]) -> Vec<RegionalCanaryAgreementView> {
    let mut views: Vec<RegionalCanaryAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_canary_agreement_view(item));
    }
    views
}

pub fn sum_regional_canary_agreement_view(view: &RegionalCanaryAgreementView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_canary_agreement("code-921", "name-179");
        let view = to_regional_canary_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_canary_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
