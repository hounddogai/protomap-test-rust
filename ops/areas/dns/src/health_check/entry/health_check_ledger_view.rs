use super::*;

use acme_kit::text;

/// How screens and exports show a health check ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerView {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub approved_text: String,
    pub visible_text: String,
    pub source: String,
    pub contact_email: String,
    pub discount: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_health_check_ledger_view(item: &HealthCheckLedger) -> HealthCheckLedgerView {
    HealthCheckLedgerView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        external_ref: item.external_ref.clone(),
        approved_text: format!("{}", item.approved),
        visible_text: format!("{}", item.visible),
        source: text::truncate(&item.source, 80),
        contact_email: item.contact_email.clone(),
        discount: format!("{:.2}", item.discount),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_ledger_views(items: &[HealthCheckLedger]) -> Vec<HealthCheckLedgerView> {
    let mut views: Vec<HealthCheckLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_ledger_view(item));
    }
    views
}

pub fn sum_health_check_ledger_view(view: &HealthCheckLedgerView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_ledger("code-686", "title-938");
        let view = to_health_check_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
