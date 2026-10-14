use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled query entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEntryView {
    pub code_text: String,
    pub label: String,
    pub duration_minutes: String,
    pub updated_at: String,
    pub namespace: String,
    pub maximum: String,
    pub discount: String,
    pub channel: String,
    pub active: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_scheduled_query_entry_view(item: &ScheduledQueryEntry) -> ScheduledQueryEntryView {
    ScheduledQueryEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        updated_at: item.updated_at.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        maximum: format!("{:.2}", item.maximum),
        discount: format!("{:.2}", item.discount),
        channel: item.channel.clone(),
        active: format!("{}", item.active),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_query_entry_views(items: &[ScheduledQueryEntry]) -> Vec<ScheduledQueryEntryView> {
    let mut views: Vec<ScheduledQueryEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_query_entry_view(item));
    }
    views
}

pub fn sum_scheduled_query_entry_view(view: &ScheduledQueryEntryView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_query_entry("code-392", "label-727");
        let view = to_scheduled_query_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_query_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
