use super::*;

/// How screens and exports show a weighted alert quota.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaView {
    pub code: String,
    pub label_text: String,
    pub region: String,
    pub amount: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_weighted_alert_quota_view(item: &WeightedAlertQuota) -> WeightedAlertQuotaView {
    WeightedAlertQuotaView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        region: item.region.clone(),
        amount: format!("{:.2}", item.amount),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_alert_quota_views(items: &[WeightedAlertQuota]) -> Vec<WeightedAlertQuotaView> {
    let mut views: Vec<WeightedAlertQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_alert_quota_view(item));
    }
    views
}

pub fn sum_weighted_alert_quota_view(view: &WeightedAlertQuotaView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_alert_quota("code-747", "label-966");
        let view = to_weighted_alert_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_alert_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
