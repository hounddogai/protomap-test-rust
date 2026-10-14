use super::*;

/// How screens and exports show a manual evidence schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleView {
    pub code: String,
    pub title: String,
    pub created_at: String,
    pub window_days: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_manual_evidence_schedule_view(item: &ManualEvidenceSchedule) -> ManualEvidenceScheduleView {
    ManualEvidenceScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        window_days: item.window_days.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_evidence_schedule_views(items: &[ManualEvidenceSchedule]) -> Vec<ManualEvidenceScheduleView> {
    let mut views: Vec<ManualEvidenceScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_evidence_schedule_view(item));
    }
    views
}

pub fn sum_manual_evidence_schedule_view(view: &ManualEvidenceScheduleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_evidence_schedule("code-753", "title-758");
        let view = to_manual_evidence_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_evidence_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
