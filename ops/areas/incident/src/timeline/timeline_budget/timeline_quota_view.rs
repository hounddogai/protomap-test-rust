use super::*;

use acme_kit::text;

/// How screens and exports show a timeline quota.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaView {
    pub code: String,
    pub name: String,
    pub host: String,
    pub rotation_days_text: String,
    pub ttl_seconds: String,
    pub archived: String,
    pub region: String,
    pub batch_size_text: String,
    pub active_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_timeline_quota_view(item: &TimelineQuota) -> TimelineQuotaView {
    TimelineQuotaView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        host: item.host.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        archived: format!("{}", item.archived),
        region: text::truncate(&item.region, 40),
        batch_size_text: item.batch_size.to_string(),
        active_text: format!("{}", item.active),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_quota_views(items: &[TimelineQuota]) -> Vec<TimelineQuotaView> {
    let mut views: Vec<TimelineQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_quota_view(item));
    }
    views
}

pub fn sum_timeline_quota_view(view: &TimelineQuotaView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_ttl_seconds as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_quota("code-75", "name-513");
        let view = to_timeline_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
