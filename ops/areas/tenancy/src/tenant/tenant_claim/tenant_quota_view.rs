use super::*;

use acme_kit::text;

/// How screens and exports show a tenant quota.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaView {
    pub code: String,
    pub label: String,
    pub margin_text: String,
    pub cipher_text: String,
    pub weight: String,
    pub country: String,
    pub amount_text: String,
    pub capacity: String,
    pub port: String,
    pub budget_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_tenant_quota_view(item: &TenantQuota) -> TenantQuotaView {
    TenantQuotaView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 120),
        margin_text: format!("{:.2}", item.margin),
        cipher_text: item.cipher.clone(),
        weight: format!("{:.2}", item.weight),
        country: item.country.clone(),
        amount_text: format!("{:.2}", item.amount),
        capacity: item.capacity.to_string(),
        port: item.port.to_string(),
        budget_text: format!("{:.2}", item.budget),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_quota_views(items: &[TenantQuota]) -> Vec<TenantQuotaView> {
    let mut views: Vec<TenantQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_quota_view(item));
    }
    views
}

pub fn sum_tenant_quota_view(view: &TenantQuotaView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_margin + parsed_weight + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_quota("code-804", "label-620");
        let view = to_tenant_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
