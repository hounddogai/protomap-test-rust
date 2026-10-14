use super::*;

use acme_kit::text;

/// How screens and exports show a tenant estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantEstimateView {
    pub code_text: String,
    pub name: String,
    pub owner_id: String,
    pub cluster: String,
    pub scope_text: String,
    pub checksum: String,
    pub channel_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_tenant_estimate_view(item: &TenantEstimate) -> TenantEstimateView {
    TenantEstimateView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        cluster: item.cluster.clone(),
        scope_text: text::truncate(&item.scope, 64),
        checksum: item.checksum.clone(),
        channel_text: item.channel.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_estimate_views(items: &[TenantEstimate]) -> Vec<TenantEstimateView> {
    let mut views: Vec<TenantEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_estimate_view(item));
    }
    views
}

pub fn sum_tenant_estimate_view(view: &TenantEstimateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
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
        let item = new_tenant_estimate("code-986", "name-706");
        let view = to_tenant_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
