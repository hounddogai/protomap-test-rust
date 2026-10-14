use super::*;

use acme_kit::text;

/// How screens and exports show a lenient dashboard ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub discount_text: String,
    pub expires_at: String,
    pub weight: String,
    pub ratio: String,
    pub priority_text: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_lenient_dashboard_ledger_view(item: &LenientDashboardLedger) -> LenientDashboardLedgerView {
    LenientDashboardLedgerView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        burst: item.burst.to_string(),
        discount_text: format!("{:.2}", item.discount),
        expires_at: item.expires_at.to_string(),
        weight: format!("{:.2}", item.weight),
        ratio: format!("{:.2}", item.ratio),
        priority_text: item.priority.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_dashboard_ledger_views(items: &[LenientDashboardLedger]) -> Vec<LenientDashboardLedgerView> {
    let mut views: Vec<LenientDashboardLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_dashboard_ledger_view(item));
    }
    views
}

pub fn sum_lenient_dashboard_ledger_view(view: &LenientDashboardLedgerView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_burst as f64 + parsed_discount + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_dashboard_ledger("code-12", "label-532");
        let view = to_lenient_dashboard_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_dashboard_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
