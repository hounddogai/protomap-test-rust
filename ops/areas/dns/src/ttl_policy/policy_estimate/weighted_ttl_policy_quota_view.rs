use super::*;

/// How screens and exports show a weighted ttl policy quota.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaView {
    pub code: String,
    pub label_text: String,
    pub locale: String,
    pub description: String,
    pub port: String,
    pub timezone: String,
    pub principal: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_weighted_ttl_policy_quota_view(item: &WeightedTtlPolicyQuota) -> WeightedTtlPolicyQuotaView {
    WeightedTtlPolicyQuotaView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        locale: item.locale.clone(),
        description: item.description.clone(),
        port: item.port.to_string(),
        timezone: item.timezone.clone(),
        principal: item.principal.clone(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_ttl_policy_quota_views(items: &[WeightedTtlPolicyQuota]) -> Vec<WeightedTtlPolicyQuotaView> {
    let mut views: Vec<WeightedTtlPolicyQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_ttl_policy_quota_view(item));
    }
    views
}

pub fn sum_weighted_ttl_policy_quota_view(view: &WeightedTtlPolicyQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_ttl_policy_quota("code-362", "label-492");
        let view = to_weighted_ttl_policy_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_ttl_policy_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
