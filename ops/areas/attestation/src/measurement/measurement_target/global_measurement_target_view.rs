use super::*;

use acme_kit::text;

/// How screens and exports show a global measurement target.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetView {
    pub code_text: String,
    pub title: String,
    pub threshold_text: String,
    pub issuer: String,
    pub ceiling: String,
    pub window_days_text: String,
    pub unit: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_global_measurement_target_view(item: &GlobalMeasurementTarget) -> GlobalMeasurementTargetView {
    GlobalMeasurementTargetView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 80),
        threshold_text: format!("{:.2}", item.threshold),
        issuer: item.issuer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        window_days_text: item.window_days.to_string(),
        unit: item.unit.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_measurement_target_views(items: &[GlobalMeasurementTarget]) -> Vec<GlobalMeasurementTargetView> {
    let mut views: Vec<GlobalMeasurementTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_measurement_target_view(item));
    }
    views
}

pub fn sum_global_measurement_target_view(view: &GlobalMeasurementTargetView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + parsed_ceiling + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_measurement_target("code-703", "title-931");
        let view = to_global_measurement_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_measurement_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
