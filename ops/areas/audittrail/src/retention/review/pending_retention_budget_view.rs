use super::*;

use acme_kit::text;

/// How screens and exports show a pending retention budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetView {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub notes_text: String,
    pub category_text: String,
    pub revision: String,
    pub region: String,
    pub window_days_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_pending_retention_budget_view(item: &PendingRetentionBudget) -> PendingRetentionBudgetView {
    PendingRetentionBudgetView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        checksum: item.checksum.clone(),
        notes_text: item.notes.clone(),
        category_text: text::truncate(&item.category, 40),
        revision: item.revision.to_string(),
        region: item.region.clone(),
        window_days_text: item.window_days.to_string(),
        country: text::truncate(&item.country, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_budget_views(items: &[PendingRetentionBudget]) -> Vec<PendingRetentionBudgetView> {
    let mut views: Vec<PendingRetentionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_budget_view(item));
    }
    views
}

pub fn sum_pending_retention_budget_view(view: &PendingRetentionBudgetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_retention_budget("code-142", "label-420");
        let view = to_pending_retention_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_retention_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
