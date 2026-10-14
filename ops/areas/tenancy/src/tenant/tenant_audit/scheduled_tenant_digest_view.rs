use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled tenant digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDigestView {
    pub code: String,
    pub label: String,
    pub approved: String,
    pub score_text: String,
    pub title: String,
    pub version_text: String,
    pub capacity: String,
    pub ends_at: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_scheduled_tenant_digest_view(item: &ScheduledTenantDigest) -> ScheduledTenantDigestView {
    ScheduledTenantDigestView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        approved: format!("{}", item.approved),
        score_text: format!("{:.2}", item.score),
        title: item.title.clone(),
        version_text: item.version.to_string(),
        capacity: item.capacity.to_string(),
        ends_at: item.ends_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_tenant_digest_views(items: &[ScheduledTenantDigest]) -> Vec<ScheduledTenantDigestView> {
    let mut views: Vec<ScheduledTenantDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_tenant_digest_view(item));
    }
    views
}

pub fn sum_scheduled_tenant_digest_view(view: &ScheduledTenantDigestView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_score + (parsed_version as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_tenant_digest("code-922", "label-580");
        let view = to_scheduled_tenant_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_tenant_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
