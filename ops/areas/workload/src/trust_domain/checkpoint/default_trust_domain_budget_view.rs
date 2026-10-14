use super::*;

use acme_kit::text;

/// How screens and exports show a default trust domain budget.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudgetView {
    pub code: String,
    pub label_text: String,
    pub limit: String,
    pub port_text: String,
    pub reviewer: String,
    pub budget: String,
    pub phone: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_default_trust_domain_budget_view(item: &DefaultTrustDomainBudget) -> DefaultTrustDomainBudgetView {
    DefaultTrustDomainBudgetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        limit: item.limit.to_string(),
        port_text: item.port.to_string(),
        reviewer: item.reviewer.clone(),
        budget: format!("{:.2}", item.budget),
        phone: text::truncate(&item.phone, 40),
        external_ref: text::truncate(&item.external_ref, 120),
        fingerprint: item.fingerprint.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_trust_domain_budget_views(items: &[DefaultTrustDomainBudget]) -> Vec<DefaultTrustDomainBudgetView> {
    let mut views: Vec<DefaultTrustDomainBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_trust_domain_budget_view(item));
    }
    views
}

pub fn sum_default_trust_domain_budget_view(view: &DefaultTrustDomainBudgetView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_port as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_trust_domain_budget("code-30", "label-262");
        let view = to_default_trust_domain_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_trust_domain_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
