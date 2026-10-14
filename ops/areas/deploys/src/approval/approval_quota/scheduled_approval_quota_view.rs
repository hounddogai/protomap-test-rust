use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled approval quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuotaView {
    pub code: String,
    pub label: String,
    pub port_text: String,
    pub amount_text: String,
    pub ends_at: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_scheduled_approval_quota_view(item: &ScheduledApprovalQuota) -> ScheduledApprovalQuotaView {
    ScheduledApprovalQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        port_text: item.port.to_string(),
        amount_text: format!("{:.2}", item.amount),
        ends_at: item.ends_at.to_string(),
        unit: text::truncate(&item.unit, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_approval_quota_views(items: &[ScheduledApprovalQuota]) -> Vec<ScheduledApprovalQuotaView> {
    let mut views: Vec<ScheduledApprovalQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_approval_quota_view(item));
    }
    views
}

pub fn sum_scheduled_approval_quota_view(view: &ScheduledApprovalQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_port as f64 + parsed_amount + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_approval_quota("code-84", "label-652");
        let view = to_scheduled_approval_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_approval_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
