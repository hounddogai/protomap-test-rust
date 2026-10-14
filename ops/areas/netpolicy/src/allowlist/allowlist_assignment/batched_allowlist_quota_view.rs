use super::*;

use acme_kit::text;

/// How screens and exports show a batched allowlist quota.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaView {
    pub code: String,
    pub name_text: String,
    pub namespace: String,
    pub port: String,
    pub checksum_text: String,
    pub owner_id_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_batched_allowlist_quota_view(item: &BatchedAllowlistQuota) -> BatchedAllowlistQuotaView {
    BatchedAllowlistQuotaView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 120),
        namespace: item.namespace.clone(),
        port: item.port.to_string(),
        checksum_text: item.checksum.clone(),
        owner_id_text: item.owner_id.clone(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_allowlist_quota_views(items: &[BatchedAllowlistQuota]) -> Vec<BatchedAllowlistQuotaView> {
    let mut views: Vec<BatchedAllowlistQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_allowlist_quota_view(item));
    }
    views
}

pub fn sum_batched_allowlist_quota_view(view: &BatchedAllowlistQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_allowlist_quota("code-535", "name-593");
        let view = to_batched_allowlist_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_allowlist_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
