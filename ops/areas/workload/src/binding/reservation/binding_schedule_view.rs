use super::*;

use acme_kit::text;

/// How screens and exports show a binding schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingScheduleView {
    pub code: String,
    pub title_text: String,
    pub window_days: String,
    pub score_text: String,
    pub channel: String,
    pub phone_text: String,
    pub unit: String,
    pub namespace: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_binding_schedule_view(item: &BindingSchedule) -> BindingScheduleView {
    BindingScheduleView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        window_days: item.window_days.to_string(),
        score_text: format!("{:.2}", item.score),
        channel: item.channel.clone(),
        phone_text: item.phone.clone(),
        unit: text::truncate(&item.unit, 120),
        namespace: text::truncate(&item.namespace, 120),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_schedule_views(items: &[BindingSchedule]) -> Vec<BindingScheduleView> {
    let mut views: Vec<BindingScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_schedule_view(item));
    }
    views
}

pub fn sum_binding_schedule_view(view: &BindingScheduleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_score + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_schedule("code-468", "title-494");
        let view = to_binding_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
