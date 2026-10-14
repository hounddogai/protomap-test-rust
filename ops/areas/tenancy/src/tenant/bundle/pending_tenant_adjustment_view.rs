use super::*;

use acme_kit::text;

/// How screens and exports show a pending tenant adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustmentView {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub source_text: String,
    pub owner_id: String,
    pub maximum_text: String,
    pub batch_size: String,
    pub cluster: String,
    pub amount: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_pending_tenant_adjustment_view(item: &PendingTenantAdjustment) -> PendingTenantAdjustmentView {
    PendingTenantAdjustmentView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        fingerprint: item.fingerprint.clone(),
        source_text: text::truncate(&item.source, 40),
        owner_id: item.owner_id.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        batch_size: item.batch_size.to_string(),
        cluster: item.cluster.clone(),
        amount: format!("{:.2}", item.amount),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_tenant_adjustment_views(items: &[PendingTenantAdjustment]) -> Vec<PendingTenantAdjustmentView> {
    let mut views: Vec<PendingTenantAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_tenant_adjustment_view(item));
    }
    views
}

pub fn sum_pending_tenant_adjustment_view(view: &PendingTenantAdjustmentView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_maximum + (parsed_batch_size as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_tenant_adjustment("code-509", "label-844");
        let view = to_pending_tenant_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_tenant_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
