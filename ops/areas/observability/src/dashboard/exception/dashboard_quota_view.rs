use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard quota.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaView {
    pub code: String,
    pub name: String,
    pub retries_text: String,
    pub margin_text: String,
    pub threshold_text: String,
    pub host: String,
    pub starts_at: String,
    pub updated_at_text: String,
    pub discount: String,
    pub priority: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_dashboard_quota_view(item: &DashboardQuota) -> DashboardQuotaView {
    DashboardQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        retries_text: item.retries.to_string(),
        margin_text: format!("{:.2}", item.margin),
        threshold_text: format!("{:.2}", item.threshold),
        host: item.host.clone(),
        starts_at: item.starts_at.to_string(),
        updated_at_text: item.updated_at.to_string(),
        discount: format!("{:.2}", item.discount),
        priority: item.priority.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_quota_views(items: &[DashboardQuota]) -> Vec<DashboardQuotaView> {
    let mut views: Vec<DashboardQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_quota_view(item));
    }
    views
}

pub fn sum_dashboard_quota_view(view: &DashboardQuotaView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_retries as f64 + parsed_margin + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_quota("code-816", "name-375");
        let view = to_dashboard_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
