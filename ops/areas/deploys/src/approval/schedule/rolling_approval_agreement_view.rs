use super::*;

use acme_kit::text;

/// How screens and exports show a rolling approval agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementView {
    pub code: String,
    pub name_text: String,
    pub amount: String,
    pub ends_at: String,
    pub window_days_text: String,
    pub quota_units_text: String,
    pub batch_size: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_rolling_approval_agreement_view(item: &RollingApprovalAgreement) -> RollingApprovalAgreementView {
    RollingApprovalAgreementView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        amount: format!("{:.2}", item.amount),
        ends_at: item.ends_at.to_string(),
        window_days_text: item.window_days.to_string(),
        quota_units_text: item.quota_units.to_string(),
        batch_size: item.batch_size.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_approval_agreement_views(items: &[RollingApprovalAgreement]) -> Vec<RollingApprovalAgreementView> {
    let mut views: Vec<RollingApprovalAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_approval_agreement_view(item));
    }
    views
}

pub fn sum_rolling_approval_agreement_view(view: &RollingApprovalAgreementView) -> Result<f64, String> {
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
        .window_days_text
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
        let item = new_rolling_approval_agreement("code-591", "name-635");
        let view = to_rolling_approval_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_approval_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
