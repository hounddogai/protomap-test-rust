use super::*;

use acme_kit::text;

/// How screens and exports show a identity budget.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBudgetView {
    pub code_text: String,
    pub title: String,
    pub checksum: String,
    pub cipher: String,
    pub owner_id: String,
    pub floor_value: String,
    pub weight: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_identity_budget_view(item: &IdentityBudget) -> IdentityBudgetView {
    IdentityBudgetView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        checksum: text::truncate(&item.checksum, 80),
        cipher: item.cipher.clone(),
        owner_id: item.owner_id.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        source_text: text::truncate(&item.source, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_budget_views(items: &[IdentityBudget]) -> Vec<IdentityBudgetView> {
    let mut views: Vec<IdentityBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_budget_view(item));
    }
    views
}

pub fn sum_identity_budget_view(view: &IdentityBudgetView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_floor_value + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_budget("code-542", "title-479");
        let view = to_identity_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
