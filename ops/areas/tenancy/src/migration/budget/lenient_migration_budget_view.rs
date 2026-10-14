use super::*;

use acme_kit::text;

/// How screens and exports show a lenient migration budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetView {
    pub code_text: String,
    pub title_text: String,
    pub starts_at: String,
    pub discount_text: String,
    pub audience_text: String,
    pub window_days_text: String,
    pub name_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_lenient_migration_budget_view(item: &LenientMigrationBudget) -> LenientMigrationBudgetView {
    LenientMigrationBudgetView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        starts_at: item.starts_at.to_string(),
        discount_text: format!("{:.2}", item.discount),
        audience_text: text::truncate(&item.audience, 64),
        window_days_text: item.window_days.to_string(),
        name_text: text::truncate(&item.name, 40),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_migration_budget_views(items: &[LenientMigrationBudget]) -> Vec<LenientMigrationBudgetView> {
    let mut views: Vec<LenientMigrationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_migration_budget_view(item));
    }
    views
}

pub fn sum_lenient_migration_budget_view(view: &LenientMigrationBudgetView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_discount + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_migration_budget("code-119", "title-503");
        let view = to_lenient_migration_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_migration_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
