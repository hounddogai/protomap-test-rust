use super::*;

use acme_kit::text;

/// How screens and exports show a legacy health check quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuotaView {
    pub code_text: String,
    pub label_text: String,
    pub owner_id_text: String,
    pub name: String,
    pub rotation_days: String,
    pub margin: String,
    pub host: String,
    pub window_days_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_legacy_health_check_quota_view(item: &LegacyHealthCheckQuota) -> LegacyHealthCheckQuotaView {
    LegacyHealthCheckQuotaView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        owner_id_text: item.owner_id.clone(),
        name: text::truncate(&item.name, 80),
        rotation_days: item.rotation_days.to_string(),
        margin: format!("{:.2}", item.margin),
        host: text::truncate(&item.host, 64),
        window_days_text: item.window_days.to_string(),
        summary: text::truncate(&item.summary, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_health_check_quota_views(items: &[LegacyHealthCheckQuota]) -> Vec<LegacyHealthCheckQuotaView> {
    let mut views: Vec<LegacyHealthCheckQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_health_check_quota_view(item));
    }
    views
}

pub fn sum_legacy_health_check_quota_view(view: &LegacyHealthCheckQuotaView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_margin + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_health_check_quota("code-621", "label-378");
        let view = to_legacy_health_check_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_health_check_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
