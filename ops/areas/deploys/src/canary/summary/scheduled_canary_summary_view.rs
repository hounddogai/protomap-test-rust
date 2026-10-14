use super::*;

/// How screens and exports show a scheduled canary summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummaryView {
    pub code: String,
    pub name_text: String,
    pub notes: String,
    pub ttl_seconds: String,
    pub principal: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_scheduled_canary_summary_view(item: &ScheduledCanarySummary) -> ScheduledCanarySummaryView {
    ScheduledCanarySummaryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        notes: item.notes.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        principal: item.principal.clone(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_canary_summary_views(items: &[ScheduledCanarySummary]) -> Vec<ScheduledCanarySummaryView> {
    let mut views: Vec<ScheduledCanarySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_canary_summary_view(item));
    }
    views
}

pub fn sum_scheduled_canary_summary_view(view: &ScheduledCanarySummaryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_canary_summary("code-556", "name-67");
        let view = to_scheduled_canary_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_canary_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
