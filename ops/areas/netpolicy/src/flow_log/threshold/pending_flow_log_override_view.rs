use super::*;

use acme_kit::text;

/// How screens and exports show a pending flow log override.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogOverrideView {
    pub code: String,
    pub name: String,
    pub title: String,
    pub ends_at: String,
    pub ttl_seconds: String,
    pub version_text: String,
    pub description_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_pending_flow_log_override_view(item: &PendingFlowLogOverride) -> PendingFlowLogOverrideView {
    PendingFlowLogOverrideView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        version_text: item.version.to_string(),
        description_text: text::truncate(&item.description, 64),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_flow_log_override_views(items: &[PendingFlowLogOverride]) -> Vec<PendingFlowLogOverrideView> {
    let mut views: Vec<PendingFlowLogOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_flow_log_override_view(item));
    }
    views
}

pub fn sum_pending_flow_log_override_view(view: &PendingFlowLogOverrideView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_ttl_seconds as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_flow_log_override("code-793", "name-421");
        let view = to_pending_flow_log_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_flow_log_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
