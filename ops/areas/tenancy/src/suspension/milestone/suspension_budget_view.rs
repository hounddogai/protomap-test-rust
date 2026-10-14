use super::*;

use acme_kit::text;

/// How screens and exports show a suspension budget.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBudgetView {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub locale: String,
    pub description: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_suspension_budget_view(item: &SuspensionBudget) -> SuspensionBudgetView {
    SuspensionBudgetView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        locale: text::truncate(&item.locale, 80),
        description: item.description.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_budget_views(items: &[SuspensionBudget]) -> Vec<SuspensionBudgetView> {
    let mut views: Vec<SuspensionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_budget_view(item));
    }
    views
}

pub fn sum_suspension_budget_view(view: &SuspensionBudgetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_budget("code-616", "label-601");
        let view = to_suspension_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
