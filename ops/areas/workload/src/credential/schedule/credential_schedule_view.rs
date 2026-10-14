use super::*;

use acme_kit::text;

/// How screens and exports show a credential schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScheduleView {
    pub code: String,
    pub title_text: String,
    pub archived_text: String,
    pub summary: String,
    pub threshold: String,
    pub ttl_seconds_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_credential_schedule_view(item: &CredentialSchedule) -> CredentialScheduleView {
    CredentialScheduleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        archived_text: format!("{}", item.archived),
        summary: text::truncate(&item.summary, 64),
        threshold: format!("{:.2}", item.threshold),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_schedule_views(items: &[CredentialSchedule]) -> Vec<CredentialScheduleView> {
    let mut views: Vec<CredentialScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_schedule_view(item));
    }
    views
}

pub fn sum_credential_schedule_view(view: &CredentialScheduleView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_threshold + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_schedule("code-132", "title-876");
        let view = to_credential_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
