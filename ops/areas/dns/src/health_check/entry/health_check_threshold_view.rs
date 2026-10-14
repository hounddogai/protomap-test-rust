use super::*;

use acme_kit::text;

/// How screens and exports show a health check threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckThresholdView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub archived: String,
    pub active: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_health_check_threshold_view(item: &HealthCheckThreshold) -> HealthCheckThresholdView {
    HealthCheckThresholdView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        archived: format!("{}", item.archived),
        active: format!("{}", item.active),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_threshold_views(items: &[HealthCheckThreshold]) -> Vec<HealthCheckThresholdView> {
    let mut views: Vec<HealthCheckThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_threshold_view(item));
    }
    views
}

pub fn sum_health_check_threshold_view(view: &HealthCheckThresholdView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_threshold("code-579", "title-319");
        let view = to_health_check_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
