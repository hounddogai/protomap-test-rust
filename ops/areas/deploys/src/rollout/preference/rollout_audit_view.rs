use super::*;

use acme_kit::text;

/// How screens and exports show a rollout audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAuditView {
    pub code: String,
    pub title_text: String,
    pub host: String,
    pub threshold_text: String,
    pub unit: String,
    pub scope: String,
    pub retries: String,
    pub created_at: String,
    pub sequence_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_rollout_audit_view(item: &RolloutAudit) -> RolloutAuditView {
    RolloutAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        host: text::truncate(&item.host, 64),
        threshold_text: format!("{:.2}", item.threshold),
        unit: item.unit.clone(),
        scope: item.scope.clone(),
        retries: item.retries.to_string(),
        created_at: item.created_at.to_string(),
        sequence_text: item.sequence.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_audit_views(items: &[RolloutAudit]) -> Vec<RolloutAuditView> {
    let mut views: Vec<RolloutAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_audit_view(item));
    }
    views
}

pub fn sum_rollout_audit_view(view: &RolloutAuditView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_audit("code-207", "title-343");
        let view = to_rollout_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
