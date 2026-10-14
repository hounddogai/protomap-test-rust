use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled isolation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationExceptionView {
    pub code: String,
    pub label_text: String,
    pub rotation_days: String,
    pub channel: String,
    pub active: String,
    pub approved: String,
    pub summary: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_scheduled_isolation_exception_view(item: &ScheduledIsolationException) -> ScheduledIsolationExceptionView {
    ScheduledIsolationExceptionView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        rotation_days: item.rotation_days.to_string(),
        channel: item.channel.clone(),
        active: format!("{}", item.active),
        approved: format!("{}", item.approved),
        summary: text::truncate(&item.summary, 80),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_isolation_exception_views(
    items: &[ScheduledIsolationException],
) -> Vec<ScheduledIsolationExceptionView> {
    let mut views: Vec<ScheduledIsolationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_isolation_exception_view(item));
    }
    views
}

pub fn sum_scheduled_isolation_exception_view(view: &ScheduledIsolationExceptionView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_isolation_exception("code-658", "label-904");
        let view = to_scheduled_isolation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_isolation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
