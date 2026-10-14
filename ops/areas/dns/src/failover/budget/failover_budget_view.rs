use super::*;

use acme_kit::text;

/// How screens and exports show a failover budget.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBudgetView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub contact_email: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_failover_budget_view(item: &FailoverBudget) -> FailoverBudgetView {
    FailoverBudgetView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        external_ref: item.external_ref.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_budget_views(items: &[FailoverBudget]) -> Vec<FailoverBudgetView> {
    let mut views: Vec<FailoverBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_budget_view(item));
    }
    views
}

pub fn sum_failover_budget_view(view: &FailoverBudgetView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_budget("code-649", "title-689");
        let view = to_failover_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
