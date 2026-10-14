use super::*;

use acme_kit::text;

/// How screens and exports show a primary suspension schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScheduleView {
    pub code_text: String,
    pub name: String,
    pub priority_text: String,
    pub archived: String,
    pub limit: String,
    pub threshold: String,
    pub summary: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_primary_suspension_schedule_view(item: &PrimarySuspensionSchedule) -> PrimarySuspensionScheduleView {
    PrimarySuspensionScheduleView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        priority_text: item.priority.to_string(),
        archived: format!("{}", item.archived),
        limit: item.limit.to_string(),
        threshold: format!("{:.2}", item.threshold),
        summary: text::truncate(&item.summary, 40),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_suspension_schedule_views(items: &[PrimarySuspensionSchedule]) -> Vec<PrimarySuspensionScheduleView> {
    let mut views: Vec<PrimarySuspensionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_suspension_schedule_view(item));
    }
    views
}

pub fn sum_primary_suspension_schedule_view(view: &PrimarySuspensionScheduleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_limit as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_suspension_schedule("code-724", "name-18");
        let view = to_primary_suspension_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_suspension_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
