use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn draft.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftView {
    pub code: String,
    pub name: String,
    pub locale_text: String,
    pub description: String,
    pub unit_text: String,
    pub expires_at: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_budget_burn_draft_view(item: &BudgetBurnDraft) -> BudgetBurnDraftView {
    BudgetBurnDraftView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 120),
        locale_text: item.locale.clone(),
        description: item.description.clone(),
        unit_text: text::truncate(&item.unit, 64),
        expires_at: item.expires_at.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_draft_views(items: &[BudgetBurnDraft]) -> Vec<BudgetBurnDraftView> {
    let mut views: Vec<BudgetBurnDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_draft_view(item));
    }
    views
}

pub fn sum_budget_burn_draft_view(view: &BudgetBurnDraftView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_draft("code-321", "name-819");
        let view = to_budget_burn_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
