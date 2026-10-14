use super::*;

use acme_kit::text;

/// How screens and exports show a health check bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundleView {
    pub code_text: String,
    pub title: String,
    pub limit: String,
    pub window_days: String,
    pub host: String,
    pub margin_text: String,
    pub discount: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_health_check_bundle_view(item: &HealthCheckBundle) -> HealthCheckBundleView {
    HealthCheckBundleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        limit: item.limit.to_string(),
        window_days: item.window_days.to_string(),
        host: item.host.clone(),
        margin_text: format!("{:.2}", item.margin),
        discount: format!("{:.2}", item.discount),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_bundle_views(items: &[HealthCheckBundle]) -> Vec<HealthCheckBundleView> {
    let mut views: Vec<HealthCheckBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_bundle_view(item));
    }
    views
}

pub fn sum_health_check_bundle_view(view: &HealthCheckBundleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_window_days as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_bundle("code-242", "title-894");
        let view = to_health_check_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
