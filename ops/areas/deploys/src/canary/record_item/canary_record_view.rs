use super::*;

use acme_kit::text;

/// How screens and exports show a canary record.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRecordView {
    pub code: String,
    pub name_text: String,
    pub created_at_text: String,
    pub issuer_text: String,
    pub tenant_id: String,
    pub archived_text: String,
    pub ttl_seconds_text: String,
    pub retries: String,
    pub margin_text: String,
    pub maximum_text: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_canary_record_view(item: &CanaryRecord) -> CanaryRecordView {
    CanaryRecordView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        created_at_text: item.created_at.to_string(),
        issuer_text: item.issuer.clone(),
        tenant_id: item.tenant_id.clone(),
        archived_text: format!("{}", item.archived),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        retries: item.retries.to_string(),
        margin_text: format!("{:.2}", item.margin),
        maximum_text: format!("{:.2}", item.maximum),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_record_views(items: &[CanaryRecord]) -> Vec<CanaryRecordView> {
    let mut views: Vec<CanaryRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_record_view(item));
    }
    views
}

pub fn sum_canary_record_view(view: &CanaryRecordView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_ttl_seconds as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_record("code-447", "name-48");
        let view = to_canary_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
