use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive health check quota.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckQuotaView {
    pub code_text: String,
    pub title_text: String,
    pub port_text: String,
    pub scope: String,
    pub external_ref_text: String,
    pub segment: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_adaptive_health_check_quota_view(item: &AdaptiveHealthCheckQuota) -> AdaptiveHealthCheckQuotaView {
    AdaptiveHealthCheckQuotaView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        port_text: item.port.to_string(),
        scope: text::truncate(&item.scope, 80),
        external_ref_text: item.external_ref.clone(),
        segment: text::truncate(&item.segment, 40),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_health_check_quota_views(items: &[AdaptiveHealthCheckQuota]) -> Vec<AdaptiveHealthCheckQuotaView> {
    let mut views: Vec<AdaptiveHealthCheckQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_health_check_quota_view(item));
    }
    views
}

pub fn sum_adaptive_health_check_quota_view(view: &AdaptiveHealthCheckQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
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
        let item = new_adaptive_health_check_quota("code-610", "title-156");
        let view = to_adaptive_health_check_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_health_check_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
