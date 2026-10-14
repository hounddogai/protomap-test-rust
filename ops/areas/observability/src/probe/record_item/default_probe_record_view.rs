use super::*;

use acme_kit::text;

/// How screens and exports show a default probe record.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeRecordView {
    pub code: String,
    pub label: String,
    pub country_text: String,
    pub revision: String,
    pub created_at_text: String,
    pub sequence: String,
    pub retries_text: String,
    pub namespace: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_default_probe_record_view(item: &DefaultProbeRecord) -> DefaultProbeRecordView {
    DefaultProbeRecordView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 64),
        country_text: item.country.clone(),
        revision: item.revision.to_string(),
        created_at_text: item.created_at.to_string(),
        sequence: item.sequence.to_string(),
        retries_text: item.retries.to_string(),
        namespace: item.namespace.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_probe_record_views(items: &[DefaultProbeRecord]) -> Vec<DefaultProbeRecordView> {
    let mut views: Vec<DefaultProbeRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_probe_record_view(item));
    }
    views
}

pub fn sum_default_probe_record_view(view: &DefaultProbeRecordView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_created_at as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_probe_record("code-263", "label-116");
        let view = to_default_probe_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_probe_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
