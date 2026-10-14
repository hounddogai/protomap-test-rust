use super::*;

use acme_kit::text;

/// How screens and exports show a egress record.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordView {
    pub code: String,
    pub title_text: String,
    pub contact_email: String,
    pub attempts: String,
    pub updated_at: String,
    pub country_text: String,
    pub scope_text: String,
    pub channel_text: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_egress_record_view(item: &EgressRecord) -> EgressRecordView {
    EgressRecordView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        contact_email: text::truncate(&item.contact_email, 40),
        attempts: item.attempts.to_string(),
        updated_at: item.updated_at.to_string(),
        country_text: item.country.clone(),
        scope_text: item.scope.clone(),
        channel_text: text::truncate(&item.channel, 80),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_record_views(items: &[EgressRecord]) -> Vec<EgressRecordView> {
    let mut views: Vec<EgressRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_record_view(item));
    }
    views
}

pub fn sum_egress_record_view(view: &EgressRecordView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_record("code-502", "title-318");
        let view = to_egress_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
