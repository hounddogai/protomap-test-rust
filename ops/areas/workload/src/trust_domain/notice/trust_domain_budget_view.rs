use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain budget.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBudgetView {
    pub code_text: String,
    pub name: String,
    pub issuer_text: String,
    pub archived_text: String,
    pub checksum_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_trust_domain_budget_view(item: &TrustDomainBudget) -> TrustDomainBudgetView {
    TrustDomainBudgetView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        issuer_text: item.issuer.clone(),
        archived_text: format!("{}", item.archived),
        checksum_text: item.checksum.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_budget_views(items: &[TrustDomainBudget]) -> Vec<TrustDomainBudgetView> {
    let mut views: Vec<TrustDomainBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_budget_view(item));
    }
    views
}

pub fn sum_trust_domain_budget_view(view: &TrustDomainBudgetView) -> Result<f64, String> {
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
        let item = new_trust_domain_budget("code-762", "name-52");
        let view = to_trust_domain_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
