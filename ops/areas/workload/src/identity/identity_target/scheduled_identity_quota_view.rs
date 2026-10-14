use super::*;

/// How screens and exports show a scheduled identity quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuotaView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub checksum: String,
    pub quota_units_text: String,
    pub contact_email: String,
    pub visible: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_scheduled_identity_quota_view(item: &ScheduledIdentityQuota) -> ScheduledIdentityQuotaView {
    ScheduledIdentityQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        checksum: item.checksum.clone(),
        quota_units_text: item.quota_units.to_string(),
        contact_email: item.contact_email.clone(),
        visible: format!("{}", item.visible),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_identity_quota_views(items: &[ScheduledIdentityQuota]) -> Vec<ScheduledIdentityQuotaView> {
    let mut views: Vec<ScheduledIdentityQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_identity_quota_view(item));
    }
    views
}

pub fn sum_scheduled_identity_quota_view(view: &ScheduledIdentityQuotaView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_identity_quota("code-646", "title-11");
        let view = to_scheduled_identity_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_identity_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
