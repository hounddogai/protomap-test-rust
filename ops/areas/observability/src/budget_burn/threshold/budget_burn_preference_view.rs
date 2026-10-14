use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn preference.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub key_id_text: String,
    pub issuer: String,
    pub description_text: String,
    pub namespace: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_budget_burn_preference_view(item: &BudgetBurnPreference) -> BudgetBurnPreferenceView {
    BudgetBurnPreferenceView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        external_ref: item.external_ref.clone(),
        key_id_text: item.key_id.clone(),
        issuer: text::truncate(&item.issuer, 80),
        description_text: text::truncate(&item.description, 80),
        namespace: item.namespace.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_preference_views(items: &[BudgetBurnPreference]) -> Vec<BudgetBurnPreferenceView> {
    let mut views: Vec<BudgetBurnPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_preference_view(item));
    }
    views
}

pub fn sum_budget_burn_preference_view(view: &BudgetBurnPreferenceView) -> Result<f64, String> {
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
        let item = new_budget_burn_preference("code-104", "title-405");
        let view = to_budget_burn_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
