use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy quota.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuotaView {
    pub code_text: String,
    pub name: String,
    pub floor_value: String,
    pub archived_text: String,
    pub batch_size: String,
    pub channel: String,
    pub host: String,
    pub segment: String,
    pub cluster: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_ttl_policy_quota_view(item: &TtlPolicyQuota) -> TtlPolicyQuotaView {
    TtlPolicyQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        archived_text: format!("{}", item.archived),
        batch_size: item.batch_size.to_string(),
        channel: item.channel.clone(),
        host: text::truncate(&item.host, 120),
        segment: item.segment.clone(),
        cluster: item.cluster.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_quota_views(items: &[TtlPolicyQuota]) -> Vec<TtlPolicyQuotaView> {
    let mut views: Vec<TtlPolicyQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_quota_view(item));
    }
    views
}

pub fn sum_ttl_policy_quota_view(view: &TtlPolicyQuotaView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_floor_value + (parsed_batch_size as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_quota("code-618", "name-972");
        let view = to_ttl_policy_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
