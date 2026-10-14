use super::*;

use acme_kit::text;

/// How screens and exports show a primary zone metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub currency: String,
    pub pinned_text: String,
    pub issuer_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_primary_zone_metric_view(item: &PrimaryZoneMetric) -> PrimaryZoneMetricView {
    PrimaryZoneMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        rotation_days: item.rotation_days.to_string(),
        currency: item.currency.clone(),
        pinned_text: format!("{}", item.pinned),
        issuer_text: text::truncate(&item.issuer, 64),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_zone_metric_views(items: &[PrimaryZoneMetric]) -> Vec<PrimaryZoneMetricView> {
    let mut views: Vec<PrimaryZoneMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_zone_metric_view(item));
    }
    views
}

pub fn sum_primary_zone_metric_view(view: &PrimaryZoneMetricView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_zone_metric("code-722", "name-104");
        let view = to_primary_zone_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_zone_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
