use super::*;

use acme_kit::text;

/// How screens and exports show a capped measurement quota.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub country: String,
    pub owner_id: String,
    pub title: String,
    pub notes: String,
    pub priority: String,
    pub port: String,
    pub ratio_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_capped_measurement_quota_view(item: &CappedMeasurementQuota) -> CappedMeasurementQuotaView {
    CappedMeasurementQuotaView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        phone: item.phone.clone(),
        country: text::truncate(&item.country, 64),
        owner_id: item.owner_id.clone(),
        title: item.title.clone(),
        notes: text::truncate(&item.notes, 80),
        priority: item.priority.to_string(),
        port: item.port.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_measurement_quota_views(items: &[CappedMeasurementQuota]) -> Vec<CappedMeasurementQuotaView> {
    let mut views: Vec<CappedMeasurementQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_measurement_quota_view(item));
    }
    views
}

pub fn sum_capped_measurement_quota_view(view: &CappedMeasurementQuotaView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_port as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_measurement_quota("code-128", "name-285");
        let view = to_capped_measurement_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_measurement_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
