use super::*;

/// How screens and exports show a tiered budget burn claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaimView {
    pub code: String,
    pub title: String,
    pub revision_text: String,
    pub source: String,
    pub unit: String,
    pub owner_id: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_tiered_budget_burn_claim_view(item: &TieredBudgetBurnClaim) -> TieredBudgetBurnClaimView {
    TieredBudgetBurnClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        revision_text: item.revision.to_string(),
        source: item.source.clone(),
        unit: item.unit.clone(),
        owner_id: item.owner_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_budget_burn_claim_views(items: &[TieredBudgetBurnClaim]) -> Vec<TieredBudgetBurnClaimView> {
    let mut views: Vec<TieredBudgetBurnClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_budget_burn_claim_view(item));
    }
    views
}

pub fn sum_tiered_budget_burn_claim_view(view: &TieredBudgetBurnClaimView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_budget_burn_claim("code-476", "title-409");
        let view = to_tiered_budget_burn_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_budget_burn_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
