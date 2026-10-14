use super::*;

use acme_kit::text;

/// How screens and exports show a usage schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageScheduleView {
    pub code: String,
    pub name_text: String,
    pub ratio_text: String,
    pub phone: String,
    pub key_id: String,
    pub window_days_text: String,
    pub category_text: String,
    pub threshold_text: String,
    pub limit: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_usage_schedule_view(item: &UsageSchedule) -> UsageScheduleView {
    UsageScheduleView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        phone: item.phone.clone(),
        key_id: item.key_id.clone(),
        window_days_text: item.window_days.to_string(),
        category_text: text::truncate(&item.category, 40),
        threshold_text: format!("{:.2}", item.threshold),
        limit: item.limit.to_string(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_schedule_views(items: &[UsageSchedule]) -> Vec<UsageScheduleView> {
    let mut views: Vec<UsageScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_schedule_view(item));
    }
    views
}

pub fn sum_usage_schedule_view(view: &UsageScheduleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ratio + (parsed_window_days as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_schedule("code-60", "name-304");
        let view = to_usage_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
