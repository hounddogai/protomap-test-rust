use super::*;

use acme_kit::text;

/// How screens and exports show a strict budget burn ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnLedgerView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub rate_text: String,
    pub external_ref: String,
    pub scope: String,
    pub source: String,
    pub batch_size: String,
    pub principal: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_strict_budget_burn_ledger_view(item: &StrictBudgetBurnLedger) -> StrictBudgetBurnLedgerView {
    StrictBudgetBurnLedgerView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        rate_text: format!("{:.2}", item.rate),
        external_ref: item.external_ref.clone(),
        scope: text::truncate(&item.scope, 80),
        source: item.source.clone(),
        batch_size: item.batch_size.to_string(),
        principal: item.principal.clone(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_budget_burn_ledger_views(items: &[StrictBudgetBurnLedger]) -> Vec<StrictBudgetBurnLedgerView> {
    let mut views: Vec<StrictBudgetBurnLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_budget_burn_ledger_view(item));
    }
    views
}

pub fn sum_strict_budget_burn_ledger_view(view: &StrictBudgetBurnLedgerView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_score + parsed_rate + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_budget_burn_ledger("code-143", "label-551");
        let view = to_strict_budget_burn_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_budget_burn_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
