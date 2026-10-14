use super::*;

use acme_kit::text;

/// How screens and exports show a redaction entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub principal: String,
    pub tenant_id: String,
    pub issuer: String,
    pub revision: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_redaction_entry_view(item: &RedactionEntry) -> RedactionEntryView {
    RedactionEntryView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        principal: text::truncate(&item.principal, 80),
        tenant_id: item.tenant_id.clone(),
        issuer: item.issuer.clone(),
        revision: item.revision.to_string(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_entry_views(items: &[RedactionEntry]) -> Vec<RedactionEntryView> {
    let mut views: Vec<RedactionEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_entry_view(item));
    }
    views
}

pub fn sum_redaction_entry_view(view: &RedactionEntryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_revision as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_entry("code-793", "title-356");
        let view = to_redaction_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
