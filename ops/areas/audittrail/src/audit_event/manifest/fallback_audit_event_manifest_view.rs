use super::*;

/// How screens and exports show a fallback audit event manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestView {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub cluster_text: String,
    pub retries_text: String,
    pub budget_text: String,
    pub scope_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_fallback_audit_event_manifest_view(item: &FallbackAuditEventManifest) -> FallbackAuditEventManifestView {
    FallbackAuditEventManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        notes: item.notes.clone(),
        cluster_text: item.cluster.clone(),
        retries_text: item.retries.to_string(),
        budget_text: format!("{:.2}", item.budget),
        scope_text: item.scope.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_audit_event_manifest_views(
    items: &[FallbackAuditEventManifest],
) -> Vec<FallbackAuditEventManifestView> {
    let mut views: Vec<FallbackAuditEventManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_audit_event_manifest_view(item));
    }
    views
}

pub fn sum_fallback_audit_event_manifest_view(view: &FallbackAuditEventManifestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_retries as f64 + parsed_budget + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_audit_event_manifest("code-672", "title-623");
        let view = to_fallback_audit_event_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_audit_event_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
