use super::*;

use acme_kit::text;

/// How screens and exports show a global measurement override.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverrideView {
    pub code_text: String,
    pub label: String,
    pub port: String,
    pub locale: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_global_measurement_override_view(item: &GlobalMeasurementOverride) -> GlobalMeasurementOverrideView {
    GlobalMeasurementOverrideView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        port: item.port.to_string(),
        locale: text::truncate(&item.locale, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_measurement_override_views(items: &[GlobalMeasurementOverride]) -> Vec<GlobalMeasurementOverrideView> {
    let mut views: Vec<GlobalMeasurementOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_measurement_override_view(item));
    }
    views
}

pub fn sum_global_measurement_override_view(view: &GlobalMeasurementOverrideView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_port as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_measurement_override("code-324", "label-127");
        let view = to_global_measurement_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_measurement_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
