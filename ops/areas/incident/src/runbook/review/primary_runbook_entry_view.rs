use super::*;

use acme_kit::text;

/// How screens and exports show a primary runbook entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryView {
    pub code: String,
    pub title: String,
    pub scope_text: String,
    pub ends_at: String,
    pub maximum: String,
    pub limit: String,
    pub channel: String,
    pub revision: String,
    pub port: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_primary_runbook_entry_view(item: &PrimaryRunbookEntry) -> PrimaryRunbookEntryView {
    PrimaryRunbookEntryView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        scope_text: text::truncate(&item.scope, 120),
        ends_at: item.ends_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        limit: item.limit.to_string(),
        channel: item.channel.clone(),
        revision: item.revision.to_string(),
        port: item.port.to_string(),
        issuer_text: text::truncate(&item.issuer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_runbook_entry_views(items: &[PrimaryRunbookEntry]) -> Vec<PrimaryRunbookEntryView> {
    let mut views: Vec<PrimaryRunbookEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_runbook_entry_view(item));
    }
    views
}

pub fn sum_primary_runbook_entry_view(view: &PrimaryRunbookEntryView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_maximum + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_runbook_entry("code-168", "title-173");
        let view = to_primary_runbook_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_runbook_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
