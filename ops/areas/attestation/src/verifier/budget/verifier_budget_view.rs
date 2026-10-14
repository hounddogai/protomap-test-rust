use super::*;

use acme_kit::text;

/// How screens and exports show a verifier budget.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetView {
    pub code: String,
    pub name: String,
    pub host_text: String,
    pub timezone: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_verifier_budget_view(item: &VerifierBudget) -> VerifierBudgetView {
    VerifierBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        host_text: text::truncate(&item.host, 64),
        timezone: item.timezone.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_budget_views(items: &[VerifierBudget]) -> Vec<VerifierBudgetView> {
    let mut views: Vec<VerifierBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_budget_view(item));
    }
    views
}

pub fn sum_verifier_budget_view(view: &VerifierBudgetView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
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
        let item = new_verifier_budget("code-745", "name-72");
        let view = to_verifier_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
