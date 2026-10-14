use super::*;

use acme_kit::text;

/// How screens and exports show a rolling rollout digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigestView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub updated_at_text: String,
    pub expires_at_text: String,
    pub category: String,
    pub contact_email: String,
    pub ttl_seconds: String,
    pub region: String,
    pub margin: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_rolling_rollout_digest_view(item: &RollingRolloutDigest) -> RollingRolloutDigestView {
    RollingRolloutDigestView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        revision: item.revision.to_string(),
        updated_at_text: item.updated_at.to_string(),
        expires_at_text: item.expires_at.to_string(),
        category: item.category.clone(),
        contact_email: item.contact_email.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        region: text::truncate(&item.region, 80),
        margin: format!("{:.2}", item.margin),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_rollout_digest_views(items: &[RollingRolloutDigest]) -> Vec<RollingRolloutDigestView> {
    let mut views: Vec<RollingRolloutDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_rollout_digest_view(item));
    }
    views
}

pub fn sum_rolling_rollout_digest_view(view: &RollingRolloutDigestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_updated_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_rollout_digest("code-959", "name-846");
        let view = to_rolling_rollout_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_rollout_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
