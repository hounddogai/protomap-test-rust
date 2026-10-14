use super::*;

use acme_kit::text;

/// How screens and exports show a timeline profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfileView {
    pub code: String,
    pub name_text: String,
    pub region: String,
    pub rotation_days_text: String,
    pub currency_text: String,
    pub floor_value_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_timeline_profile_view(item: &TimelineProfile) -> TimelineProfileView {
    TimelineProfileView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        region: item.region.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        currency_text: item.currency.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_profile_views(items: &[TimelineProfile]) -> Vec<TimelineProfileView> {
    let mut views: Vec<TimelineProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_profile_view(item));
    }
    views
}

pub fn sum_timeline_profile_view(view: &TimelineProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_floor_value + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_profile("code-832", "name-761");
        let view = to_timeline_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
