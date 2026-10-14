use super::*;

use acme_kit::text;

/// How screens and exports show a rolling zone quota.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuotaView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub threshold: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_rolling_zone_quota_view(item: &RollingZoneQuota) -> RollingZoneQuotaView {
    RollingZoneQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: text::truncate(&item.summary, 80),
        threshold: format!("{:.2}", item.threshold),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_zone_quota_views(items: &[RollingZoneQuota]) -> Vec<RollingZoneQuotaView> {
    let mut views: Vec<RollingZoneQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_zone_quota_view(item));
    }
    views
}

pub fn sum_rolling_zone_quota_view(view: &RollingZoneQuotaView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_zone_quota("code-461", "name-508");
        let view = to_rolling_zone_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_zone_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
