use super::*;

use acme_kit::text;

/// How screens and exports show a grant quota.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantQuotaView {
    pub code: String,
    pub label: String,
    pub priority_text: String,
    pub attempts: String,
    pub ttl_seconds_text: String,
    pub amount: String,
    pub capacity: String,
    pub archived_text: String,
    pub floor_value_text: String,
    pub ends_at: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_grant_quota_view(item: &GrantQuota) -> GrantQuotaView {
    GrantQuotaView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        priority_text: item.priority.to_string(),
        attempts: item.attempts.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        amount: format!("{:.2}", item.amount),
        capacity: item.capacity.to_string(),
        archived_text: format!("{}", item.archived),
        floor_value_text: format!("{:.2}", item.floor_value),
        ends_at: item.ends_at.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_quota_views(items: &[GrantQuota]) -> Vec<GrantQuotaView> {
    let mut views: Vec<GrantQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_quota_view(item));
    }
    views
}

pub fn sum_grant_quota_view(view: &GrantQuotaView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_attempts as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_quota("code-9", "label-548");
        let view = to_grant_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
