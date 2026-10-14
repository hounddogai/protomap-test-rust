use super::*;

use acme_kit::text;

/// How screens and exports show a policy exception.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyExceptionView {
    pub code_text: String,
    pub label: String,
    pub budget: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_policy_exception_view(item: &PolicyException) -> PolicyExceptionView {
    PolicyExceptionView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        budget: format!("{:.2}", item.budget),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_exception_views(items: &[PolicyException]) -> Vec<PolicyExceptionView> {
    let mut views: Vec<PolicyExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_exception_view(item));
    }
    views
}

pub fn sum_policy_exception_view(view: &PolicyExceptionView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_budget + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_exception("code-347", "label-336");
        let view = to_policy_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
