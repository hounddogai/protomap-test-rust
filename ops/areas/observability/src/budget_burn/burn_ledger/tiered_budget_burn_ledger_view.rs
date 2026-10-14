use super::*;

use acme_kit::text;

/// How screens and exports show a tiered budget burn ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub locale: String,
    pub ratio: String,
    pub score: String,
    pub limit_text: String,
    pub margin: String,
    pub rotation_days: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_tiered_budget_burn_ledger_view(item: &TieredBudgetBurnLedger) -> TieredBudgetBurnLedgerView {
    TieredBudgetBurnLedgerView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 40),
        pinned: format!("{}", item.pinned),
        locale: item.locale.clone(),
        ratio: format!("{:.2}", item.ratio),
        score: format!("{:.2}", item.score),
        limit_text: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        rotation_days: item.rotation_days.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_budget_burn_ledger_views(items: &[TieredBudgetBurnLedger]) -> Vec<TieredBudgetBurnLedgerView> {
    let mut views: Vec<TieredBudgetBurnLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_budget_burn_ledger_view(item));
    }
    views
}

pub fn sum_tiered_budget_burn_ledger_view(view: &TieredBudgetBurnLedgerView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ratio + parsed_score + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_budget_burn_ledger("code-511", "title-499");
        let view = to_tiered_budget_burn_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_budget_burn_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
