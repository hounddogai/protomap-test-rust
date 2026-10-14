use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled silence rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupView {
    pub code_text: String,
    pub name_text: String,
    pub tenant_id: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub minimum: String,
    pub rate: String,
    pub rotation_days_text: String,
    pub duration_minutes: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_scheduled_silence_rollup_view(item: &ScheduledSilenceRollup) -> ScheduledSilenceRollupView {
    ScheduledSilenceRollupView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        title: item.title.clone(),
        checksum: text::truncate(&item.checksum, 64),
        fingerprint: item.fingerprint.clone(),
        minimum: format!("{:.2}", item.minimum),
        rate: format!("{:.2}", item.rate),
        rotation_days_text: item.rotation_days.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_silence_rollup_views(items: &[ScheduledSilenceRollup]) -> Vec<ScheduledSilenceRollupView> {
    let mut views: Vec<ScheduledSilenceRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_silence_rollup_view(item));
    }
    views
}

pub fn sum_scheduled_silence_rollup_view(view: &ScheduledSilenceRollupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_minimum + parsed_rate + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_silence_rollup("code-89", "name-509");
        let view = to_scheduled_silence_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_silence_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
