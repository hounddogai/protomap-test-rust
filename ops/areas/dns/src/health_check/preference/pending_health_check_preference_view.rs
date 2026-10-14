use super::*;

use acme_kit::text;

/// How screens and exports show a pending health check preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckPreferenceView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub audience: String,
    pub pinned: String,
    pub contact_email_text: String,
    pub updated_at_text: String,
    pub checksum: String,
    pub cluster: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_pending_health_check_preference_view(
    item: &PendingHealthCheckPreference,
) -> PendingHealthCheckPreferenceView {
    PendingHealthCheckPreferenceView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        region: text::truncate(&item.region, 120),
        audience: item.audience.clone(),
        pinned: format!("{}", item.pinned),
        contact_email_text: item.contact_email.clone(),
        updated_at_text: item.updated_at.to_string(),
        checksum: item.checksum.clone(),
        cluster: text::truncate(&item.cluster, 80),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_health_check_preference_views(
    items: &[PendingHealthCheckPreference],
) -> Vec<PendingHealthCheckPreferenceView> {
    let mut views: Vec<PendingHealthCheckPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_health_check_preference_view(item));
    }
    views
}

pub fn sum_pending_health_check_preference_view(view: &PendingHealthCheckPreferenceView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_health_check_preference("code-252", "name-919");
        let view = to_pending_health_check_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_health_check_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
