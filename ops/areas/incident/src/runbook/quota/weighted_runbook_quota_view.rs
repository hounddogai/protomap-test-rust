use super::*;

use acme_kit::text;

/// How screens and exports show a weighted runbook quota.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookQuotaView {
    pub code: String,
    pub name: String,
    pub host_text: String,
    pub scope: String,
    pub segment: String,
    pub maximum: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_weighted_runbook_quota_view(item: &WeightedRunbookQuota) -> WeightedRunbookQuotaView {
    WeightedRunbookQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        host_text: text::truncate(&item.host, 80),
        scope: text::truncate(&item.scope, 40),
        segment: text::truncate(&item.segment, 120),
        maximum: format!("{:.2}", item.maximum),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_runbook_quota_views(items: &[WeightedRunbookQuota]) -> Vec<WeightedRunbookQuotaView> {
    let mut views: Vec<WeightedRunbookQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_runbook_quota_view(item));
    }
    views
}

pub fn sum_weighted_runbook_quota_view(view: &WeightedRunbookQuotaView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_maximum + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_runbook_quota("code-937", "name-359");
        let view = to_weighted_runbook_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_runbook_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
