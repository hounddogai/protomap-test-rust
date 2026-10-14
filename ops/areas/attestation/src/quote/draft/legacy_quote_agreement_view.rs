use super::*;

use acme_kit::text;

/// How screens and exports show a legacy quote agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteAgreementView {
    pub code: String,
    pub name: String,
    pub timezone_text: String,
    pub budget_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_legacy_quote_agreement_view(item: &LegacyQuoteAgreement) -> LegacyQuoteAgreementView {
    LegacyQuoteAgreementView {
        code: item.code.clone(),
        name: item.name.clone(),
        timezone_text: text::truncate(&item.timezone, 64),
        budget_text: format!("{:.2}", item.budget),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_quote_agreement_views(items: &[LegacyQuoteAgreement]) -> Vec<LegacyQuoteAgreementView> {
    let mut views: Vec<LegacyQuoteAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_quote_agreement_view(item));
    }
    views
}

pub fn sum_legacy_quote_agreement_view(view: &LegacyQuoteAgreementView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_quote_agreement("code-549", "name-262");
        let view = to_legacy_quote_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_quote_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
