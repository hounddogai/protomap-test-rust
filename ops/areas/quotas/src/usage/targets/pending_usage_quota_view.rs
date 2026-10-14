use super::*;

/// How screens and exports show a pending usage quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageQuotaView {
    pub code: String,
    pub label: String,
    pub revision: String,
    pub burst: String,
    pub rotation_days_text: String,
    pub quantity: String,
    pub channel: String,
    pub sequence_text: String,
    pub floor_value_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_pending_usage_quota_view(item: &PendingUsageQuota) -> PendingUsageQuotaView {
    PendingUsageQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        burst: item.burst.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        quantity: item.quantity.to_string(),
        channel: item.channel.clone(),
        sequence_text: item.sequence.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_usage_quota_views(items: &[PendingUsageQuota]) -> Vec<PendingUsageQuotaView> {
    let mut views: Vec<PendingUsageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_usage_quota_view(item));
    }
    views
}

pub fn sum_pending_usage_quota_view(view: &PendingUsageQuotaView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_burst as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_usage_quota("code-882", "label-622");
        let view = to_pending_usage_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_usage_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
