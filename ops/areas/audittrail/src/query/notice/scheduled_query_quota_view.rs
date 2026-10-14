use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled query quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryQuotaView {
    pub code: String,
    pub title: String,
    pub rate_text: String,
    pub margin: String,
    pub active: String,
    pub score: String,
    pub cluster_text: String,
    pub key_id: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_scheduled_query_quota_view(item: &ScheduledQueryQuota) -> ScheduledQueryQuotaView {
    ScheduledQueryQuotaView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        rate_text: format!("{:.2}", item.rate),
        margin: format!("{:.2}", item.margin),
        active: format!("{}", item.active),
        score: format!("{:.2}", item.score),
        cluster_text: text::truncate(&item.cluster, 120),
        key_id: item.key_id.clone(),
        region_text: text::truncate(&item.region, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_query_quota_views(items: &[ScheduledQueryQuota]) -> Vec<ScheduledQueryQuotaView> {
    let mut views: Vec<ScheduledQueryQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_query_quota_view(item));
    }
    views
}

pub fn sum_scheduled_query_quota_view(view: &ScheduledQueryQuotaView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_rate + parsed_margin + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_query_quota("code-472", "title-196");
        let view = to_scheduled_query_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_query_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
