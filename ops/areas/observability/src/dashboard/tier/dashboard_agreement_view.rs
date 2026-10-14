use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreementView {
    pub code_text: String,
    pub label: String,
    pub amount: String,
    pub ends_at: String,
    pub contact_email: String,
    pub pinned: String,
    pub cipher: String,
    pub host: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_dashboard_agreement_view(item: &DashboardAgreement) -> DashboardAgreementView {
    DashboardAgreementView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        ends_at: item.ends_at.to_string(),
        contact_email: item.contact_email.clone(),
        pinned: format!("{}", item.pinned),
        cipher: text::truncate(&item.cipher, 64),
        host: item.host.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_agreement_views(items: &[DashboardAgreement]) -> Vec<DashboardAgreementView> {
    let mut views: Vec<DashboardAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_agreement_view(item));
    }
    views
}

pub fn sum_dashboard_agreement_view(view: &DashboardAgreementView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_amount + (parsed_ends_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_agreement("code-676", "label-225");
        let view = to_dashboard_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
