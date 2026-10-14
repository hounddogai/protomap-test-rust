use super::*;

use acme_kit::text;

/// How screens and exports show a timeline cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycleView {
    pub code: String,
    pub name: String,
    pub approved_text: String,
    pub ttl_seconds: String,
    pub window_days_text: String,
    pub checksum: String,
    pub threshold: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_timeline_cycle_view(item: &TimelineCycle) -> TimelineCycleView {
    TimelineCycleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        approved_text: format!("{}", item.approved),
        ttl_seconds: item.ttl_seconds.to_string(),
        window_days_text: item.window_days.to_string(),
        checksum: item.checksum.clone(),
        threshold: format!("{:.2}", item.threshold),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_cycle_views(items: &[TimelineCycle]) -> Vec<TimelineCycleView> {
    let mut views: Vec<TimelineCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_cycle_view(item));
    }
    views
}

pub fn sum_timeline_cycle_view(view: &TimelineCycleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_window_days as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_cycle("code-178", "name-935");
        let view = to_timeline_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
