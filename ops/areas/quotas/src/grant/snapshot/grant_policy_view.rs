use super::*;

use acme_kit::text;

/// How screens and exports show a grant policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyView {
    pub code: String,
    pub name: String,
    pub ceiling_text: String,
    pub cluster: String,
    pub discount: String,
    pub budget: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_grant_policy_view(item: &GrantPolicy) -> GrantPolicyView {
    GrantPolicyView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        cluster: item.cluster.clone(),
        discount: format!("{:.2}", item.discount),
        budget: format!("{:.2}", item.budget),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_policy_views(items: &[GrantPolicy]) -> Vec<GrantPolicyView> {
    let mut views: Vec<GrantPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_policy_view(item));
    }
    views
}

pub fn sum_grant_policy_view(view: &GrantPolicyView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ceiling + parsed_discount + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_policy("code-95", "name-898");
        let view = to_grant_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
