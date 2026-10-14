use super::*;

use acme_kit::text;

/// How screens and exports show a zone metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetricView {
    pub code: String,
    pub name: String,
    pub category_text: String,
    pub country: String,
    pub rate: String,
    pub port: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_zone_metric_view(item: &ZoneMetric) -> ZoneMetricView {
    ZoneMetricView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        category_text: item.category.clone(),
        country: item.country.clone(),
        rate: format!("{:.2}", item.rate),
        port: item.port.to_string(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_metric_views(items: &[ZoneMetric]) -> Vec<ZoneMetricView> {
    let mut views: Vec<ZoneMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_metric_view(item));
    }
    views
}

pub fn sum_zone_metric_view(view: &ZoneMetricView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_rate + (parsed_port as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_metric("code-531", "name-646");
        let view = to_zone_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
