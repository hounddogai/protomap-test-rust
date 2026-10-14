use super::*;

use acme_kit::text;

/// How screens and exports show a egress policy.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyView {
    pub code_text: String,
    pub label: String,
    pub contact_email: String,
    pub issuer: String,
    pub currency: String,
    pub retries: String,
    pub description: String,
    pub summary: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_egress_policy_view(item: &EgressPolicy) -> EgressPolicyView {
    EgressPolicyView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        issuer: text::truncate(&item.issuer, 80),
        currency: item.currency.clone(),
        retries: item.retries.to_string(),
        description: item.description.clone(),
        summary: item.summary.clone(),
        title: text::truncate(&item.title, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_policy_views(items: &[EgressPolicy]) -> Vec<EgressPolicyView> {
    let mut views: Vec<EgressPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_policy_view(item));
    }
    views
}

pub fn sum_egress_policy_view(view: &EgressPolicyView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_policy("code-832", "label-194");
        let view = to_egress_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
