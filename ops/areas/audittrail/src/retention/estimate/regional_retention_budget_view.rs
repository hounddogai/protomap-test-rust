use super::*;

/// How screens and exports show a regional retention budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionBudgetView {
    pub code_text: String,
    pub name_text: String,
    pub minimum: String,
    pub created_at_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_regional_retention_budget_view(item: &RegionalRetentionBudget) -> RegionalRetentionBudgetView {
    RegionalRetentionBudgetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        created_at_text: item.created_at.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_retention_budget_views(items: &[RegionalRetentionBudget]) -> Vec<RegionalRetentionBudgetView> {
    let mut views: Vec<RegionalRetentionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_retention_budget_view(item));
    }
    views
}

pub fn sum_regional_retention_budget_view(view: &RegionalRetentionBudgetView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_minimum + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_retention_budget("code-4", "name-224");
        let view = to_regional_retention_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_retention_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
