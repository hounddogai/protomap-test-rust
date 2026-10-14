use super::*;

use acme_kit::text;

/// How screens and exports show a alert preference.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreferenceView {
    pub code_text: String,
    pub title: String,
    pub segment: String,
    pub threshold_text: String,
    pub port_text: String,
    pub active_text: String,
    pub locale: String,
    pub window_days: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_alert_preference_view(item: &AlertPreference) -> AlertPreferenceView {
    AlertPreferenceView {
        code_text: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 40),
        segment: item.segment.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        port_text: item.port.to_string(),
        active_text: format!("{}", item.active),
        locale: text::truncate(&item.locale, 120),
        window_days: item.window_days.to_string(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_preference_views(items: &[AlertPreference]) -> Vec<AlertPreferenceView> {
    let mut views: Vec<AlertPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_preference_view(item));
    }
    views
}

pub fn sum_alert_preference_view(view: &AlertPreferenceView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + (parsed_port as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_preference("code-237", "title-748");
        let view = to_alert_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
