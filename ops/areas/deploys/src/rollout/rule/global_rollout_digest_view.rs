use super::*;

use acme_kit::text;

/// How screens and exports show a global rollout digest.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutDigestView {
    pub code_text: String,
    pub name_text: String,
    pub tenant_id: String,
    pub revision_text: String,
    pub margin: String,
    pub country: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_global_rollout_digest_view(item: &GlobalRolloutDigest) -> GlobalRolloutDigestView {
    GlobalRolloutDigestView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        revision_text: item.revision.to_string(),
        margin: format!("{:.2}", item.margin),
        country: text::truncate(&item.country, 40),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_rollout_digest_views(items: &[GlobalRolloutDigest]) -> Vec<GlobalRolloutDigestView> {
    let mut views: Vec<GlobalRolloutDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_rollout_digest_view(item));
    }
    views
}

pub fn sum_global_rollout_digest_view(view: &GlobalRolloutDigestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_revision as f64 + parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_rollout_digest("code-449", "name-828");
        let view = to_global_rollout_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_rollout_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
