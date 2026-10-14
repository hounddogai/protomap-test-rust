use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled grant digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantDigestView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub limit_text: String,
    pub revision: String,
    pub notes: String,
    pub contact_email: String,
    pub channel_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_scheduled_grant_digest_view(item: &ScheduledGrantDigest) -> ScheduledGrantDigestView {
    ScheduledGrantDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        limit_text: item.limit.to_string(),
        revision: item.revision.to_string(),
        notes: item.notes.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        channel_text: item.channel.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_grant_digest_views(items: &[ScheduledGrantDigest]) -> Vec<ScheduledGrantDigestView> {
    let mut views: Vec<ScheduledGrantDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_grant_digest_view(item));
    }
    views
}

pub fn sum_scheduled_grant_digest_view(view: &ScheduledGrantDigestView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_limit as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_grant_digest("code-627", "name-752");
        let view = to_scheduled_grant_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_grant_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
