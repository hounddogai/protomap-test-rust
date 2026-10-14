use super::*;

use acme_kit::text;

/// How screens and exports show a pending query quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaView {
    pub code: String,
    pub title: String,
    pub version: String,
    pub rate_text: String,
    pub retries: String,
    pub principal: String,
    pub quota_units: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_pending_query_quota_view(item: &PendingQueryQuota) -> PendingQueryQuotaView {
    PendingQueryQuotaView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        version: item.version.to_string(),
        rate_text: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        principal: item.principal.clone(),
        quota_units: item.quota_units.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_query_quota_views(items: &[PendingQueryQuota]) -> Vec<PendingQueryQuotaView> {
    let mut views: Vec<PendingQueryQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_query_quota_view(item));
    }
    views
}

pub fn sum_pending_query_quota_view(view: &PendingQueryQuotaView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_version as f64 + parsed_rate + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_query_quota("code-923", "title-94");
        let view = to_pending_query_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_query_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
