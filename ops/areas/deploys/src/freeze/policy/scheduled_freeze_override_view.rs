use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled freeze override.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeOverrideView {
    pub code: String,
    pub label: String,
    pub title_text: String,
    pub grace_days: String,
    pub window_days_text: String,
    pub external_ref_text: String,
    pub checksum_text: String,
    pub floor_value: String,
    pub starts_at: String,
    pub rate: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_scheduled_freeze_override_view(item: &ScheduledFreezeOverride) -> ScheduledFreezeOverrideView {
    ScheduledFreezeOverrideView {
        code: item.code.clone(),
        label: item.label.clone(),
        title_text: text::truncate(&item.title, 80),
        grace_days: item.grace_days.to_string(),
        window_days_text: item.window_days.to_string(),
        external_ref_text: item.external_ref.clone(),
        checksum_text: item.checksum.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        starts_at: item.starts_at.to_string(),
        rate: format!("{:.2}", item.rate),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_freeze_override_views(items: &[ScheduledFreezeOverride]) -> Vec<ScheduledFreezeOverrideView> {
    let mut views: Vec<ScheduledFreezeOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_freeze_override_view(item));
    }
    views
}

pub fn sum_scheduled_freeze_override_view(view: &ScheduledFreezeOverrideView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_window_days as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_freeze_override("code-684", "label-417");
        let view = to_scheduled_freeze_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_freeze_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
