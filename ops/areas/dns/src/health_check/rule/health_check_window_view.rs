use super::*;

use acme_kit::text;

/// How screens and exports show a health check window.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowView {
    pub code: String,
    pub label: String,
    pub ratio_text: String,
    pub starts_at: String,
    pub active: String,
    pub namespace: String,
    pub ceiling: String,
    pub region: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_health_check_window_view(item: &HealthCheckWindow) -> HealthCheckWindowView {
    HealthCheckWindowView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        starts_at: item.starts_at.to_string(),
        active: format!("{}", item.active),
        namespace: item.namespace.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        region: item.region.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_window_views(items: &[HealthCheckWindow]) -> Vec<HealthCheckWindowView> {
    let mut views: Vec<HealthCheckWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_window_view(item));
    }
    views
}

pub fn sum_health_check_window_view(view: &HealthCheckWindowView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ratio + (parsed_starts_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_window("code-975", "label-613");
        let view = to_health_check_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
