use super::*;

use acme_kit::text;

/// How screens and exports show a limit ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedgerView {
    pub code: String,
    pub title: String,
    pub amount: String,
    pub score: String,
    pub audience: String,
    pub grace_days: String,
    pub limit_text: String,
    pub cluster: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_limit_ledger_view(item: &LimitLedger) -> LimitLedgerView {
    LimitLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount: format!("{:.2}", item.amount),
        score: format!("{:.2}", item.score),
        audience: item.audience.clone(),
        grace_days: item.grace_days.to_string(),
        limit_text: item.limit.to_string(),
        cluster: text::truncate(&item.cluster, 80),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_ledger_views(items: &[LimitLedger]) -> Vec<LimitLedgerView> {
    let mut views: Vec<LimitLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_ledger_view(item));
    }
    views
}

pub fn sum_limit_ledger_view(view: &LimitLedgerView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_amount + parsed_score + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_ledger("code-521", "title-286");
        let view = to_limit_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
