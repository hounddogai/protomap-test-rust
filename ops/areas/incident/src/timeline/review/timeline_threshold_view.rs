use super::*;

use acme_kit::text;

/// How screens and exports show a timeline threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThresholdView {
    pub code: String,
    pub title: String,
    pub port_text: String,
    pub discount: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_timeline_threshold_view(item: &TimelineThreshold) -> TimelineThresholdView {
    TimelineThresholdView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        port_text: item.port.to_string(),
        discount: format!("{:.2}", item.discount),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_threshold_views(items: &[TimelineThreshold]) -> Vec<TimelineThresholdView> {
    let mut views: Vec<TimelineThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_threshold_view(item));
    }
    views
}

pub fn sum_timeline_threshold_view(view: &TimelineThresholdView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_port as f64 + parsed_discount + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_threshold("code-761", "title-78");
        let view = to_timeline_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
