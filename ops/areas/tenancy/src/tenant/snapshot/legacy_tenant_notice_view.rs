use super::*;

use acme_kit::text;

/// How screens and exports show a legacy tenant notice.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantNoticeView {
    pub code: String,
    pub title: String,
    pub version_text: String,
    pub pinned_text: String,
    pub rate: String,
    pub window_days: String,
    pub duration_minutes: String,
    pub port_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_legacy_tenant_notice_view(item: &LegacyTenantNotice) -> LegacyTenantNoticeView {
    LegacyTenantNoticeView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        version_text: item.version.to_string(),
        pinned_text: format!("{}", item.pinned),
        rate: format!("{:.2}", item.rate),
        window_days: item.window_days.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        port_text: item.port.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_tenant_notice_views(items: &[LegacyTenantNotice]) -> Vec<LegacyTenantNoticeView> {
    let mut views: Vec<LegacyTenantNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_tenant_notice_view(item));
    }
    views
}

pub fn sum_legacy_tenant_notice_view(view: &LegacyTenantNoticeView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_version as f64 + parsed_rate + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_tenant_notice("code-827", "title-30");
        let view = to_legacy_tenant_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_tenant_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
