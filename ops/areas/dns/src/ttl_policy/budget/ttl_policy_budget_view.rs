use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy budget.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBudgetView {
    pub code_text: String,
    pub title: String,
    pub region: String,
    pub ratio: String,
    pub namespace: String,
    pub cluster: String,
    pub currency: String,
    pub checksum: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_ttl_policy_budget_view(item: &TtlPolicyBudget) -> TtlPolicyBudgetView {
    TtlPolicyBudgetView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        region: text::truncate(&item.region, 64),
        ratio: format!("{:.2}", item.ratio),
        namespace: text::truncate(&item.namespace, 40),
        cluster: text::truncate(&item.cluster, 40),
        currency: item.currency.clone(),
        checksum: text::truncate(&item.checksum, 80),
        principal: text::truncate(&item.principal, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_budget_views(items: &[TtlPolicyBudget]) -> Vec<TtlPolicyBudgetView> {
    let mut views: Vec<TtlPolicyBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_budget_view(item));
    }
    views
}

pub fn sum_ttl_policy_budget_view(view: &TtlPolicyBudgetView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_budget("code-435", "title-419");
        let view = to_ttl_policy_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
