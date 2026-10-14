use super::*;

/// How screens and exports show a redaction summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSummaryView {
    pub code: String,
    pub title: String,
    pub window_days: String,
    pub priority: String,
    pub ends_at: String,
    pub ttl_seconds_text: String,
    pub revision: String,
    pub amount_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_redaction_summary_view(item: &RedactionSummary) -> RedactionSummaryView {
    RedactionSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        priority: item.priority.to_string(),
        ends_at: item.ends_at.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        revision: item.revision.to_string(),
        amount_text: format!("{:.2}", item.amount),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_summary_views(items: &[RedactionSummary]) -> Vec<RedactionSummaryView> {
    let mut views: Vec<RedactionSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_summary_view(item));
    }
    views
}

pub fn sum_redaction_summary_view(view: &RedactionSummaryView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_priority as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_summary("code-553", "title-455");
        let view = to_redaction_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
