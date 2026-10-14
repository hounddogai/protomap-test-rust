use super::*;

/// How screens and exports show a tiered budget burn profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileView {
    pub code_text: String,
    pub title_text: String,
    pub cipher: String,
    pub host: String,
    pub principal: String,
    pub namespace: String,
    pub budget_text: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_tiered_budget_burn_profile_view(item: &TieredBudgetBurnProfile) -> TieredBudgetBurnProfileView {
    TieredBudgetBurnProfileView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        cipher: item.cipher.clone(),
        host: item.host.clone(),
        principal: item.principal.clone(),
        namespace: item.namespace.clone(),
        budget_text: format!("{:.2}", item.budget),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_budget_burn_profile_views(items: &[TieredBudgetBurnProfile]) -> Vec<TieredBudgetBurnProfileView> {
    let mut views: Vec<TieredBudgetBurnProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_budget_burn_profile_view(item));
    }
    views
}

pub fn sum_tiered_budget_burn_profile_view(view: &TieredBudgetBurnProfileView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_budget_burn_profile("code-536", "title-387");
        let view = to_tiered_budget_burn_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_budget_burn_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
