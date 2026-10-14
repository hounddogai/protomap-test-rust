use super::*;

/// How screens and exports show a scheduled usage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEntryView {
    pub code_text: String,
    pub label: String,
    pub visible_text: String,
    pub expires_at: String,
    pub grace_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_scheduled_usage_entry_view(item: &ScheduledUsageEntry) -> ScheduledUsageEntryView {
    ScheduledUsageEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        visible_text: format!("{}", item.visible),
        expires_at: item.expires_at.to_string(),
        grace_days: item.grace_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_usage_entry_views(items: &[ScheduledUsageEntry]) -> Vec<ScheduledUsageEntryView> {
    let mut views: Vec<ScheduledUsageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_usage_entry_view(item));
    }
    views
}

pub fn sum_scheduled_usage_entry_view(view: &ScheduledUsageEntryView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_grace_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_usage_entry("code-696", "label-446");
        let view = to_scheduled_usage_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_usage_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
