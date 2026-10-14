use super::*;

use acme_kit::text;

/// How screens and exports show a regional retention schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionScheduleView {
    pub code: String,
    pub title: String,
    pub created_at_text: String,
    pub currency_text: String,
    pub grace_days: String,
    pub cluster_text: String,
    pub duration_minutes: String,
    pub window_days: String,
    pub unit: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_regional_retention_schedule_view(item: &RegionalRetentionSchedule) -> RegionalRetentionScheduleView {
    RegionalRetentionScheduleView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        created_at_text: item.created_at.to_string(),
        currency_text: item.currency.clone(),
        grace_days: item.grace_days.to_string(),
        cluster_text: text::truncate(&item.cluster, 40),
        duration_minutes: item.duration_minutes.to_string(),
        window_days: item.window_days.to_string(),
        unit: item.unit.clone(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_retention_schedule_views(items: &[RegionalRetentionSchedule]) -> Vec<RegionalRetentionScheduleView> {
    let mut views: Vec<RegionalRetentionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_retention_schedule_view(item));
    }
    views
}

pub fn sum_regional_retention_schedule_view(view: &RegionalRetentionScheduleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_grace_days as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_retention_schedule("code-714", "title-134");
        let view = to_regional_retention_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_retention_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
