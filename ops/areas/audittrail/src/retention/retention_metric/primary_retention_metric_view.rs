use super::*;

use acme_kit::text;

/// How screens and exports show a primary retention metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricView {
    pub code_text: String,
    pub name: String,
    pub quota_units: String,
    pub maximum: String,
    pub phone: String,
    pub pinned: String,
    pub country: String,
    pub checksum_text: String,
    pub created_at: String,
    pub budget_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_primary_retention_metric_view(item: &PrimaryRetentionMetric) -> PrimaryRetentionMetricView {
    PrimaryRetentionMetricView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        quota_units: item.quota_units.to_string(),
        maximum: format!("{:.2}", item.maximum),
        phone: item.phone.clone(),
        pinned: format!("{}", item.pinned),
        country: text::truncate(&item.country, 80),
        checksum_text: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        budget_text: format!("{:.2}", item.budget),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_retention_metric_views(items: &[PrimaryRetentionMetric]) -> Vec<PrimaryRetentionMetricView> {
    let mut views: Vec<PrimaryRetentionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_retention_metric_view(item));
    }
    views
}

pub fn sum_primary_retention_metric_view(view: &PrimaryRetentionMetricView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_maximum + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_retention_metric("code-345", "name-253");
        let view = to_primary_retention_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_retention_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
