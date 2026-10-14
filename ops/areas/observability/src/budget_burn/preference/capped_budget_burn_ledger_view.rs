use super::*;

/// How screens and exports show a capped budget burn ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerView {
    pub code: String,
    pub title: String,
    pub updated_at_text: String,
    pub namespace_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_ledger_view(item: &CappedBudgetBurnLedger) -> CappedBudgetBurnLedgerView {
    CappedBudgetBurnLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        updated_at_text: item.updated_at.to_string(),
        namespace_text: item.namespace.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_ledger_views(items: &[CappedBudgetBurnLedger]) -> Vec<CappedBudgetBurnLedgerView> {
    let mut views: Vec<CappedBudgetBurnLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_ledger_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_ledger_view(view: &CappedBudgetBurnLedgerView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_budget_burn_ledger("code-718", "title-748");
        let view = to_capped_budget_burn_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_budget_burn_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
