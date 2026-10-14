use super::*;

/// How screens and exports show a adaptive audit event score.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventScoreView {
    pub code: String,
    pub name: String,
    pub source: String,
    pub checksum: String,
    pub issuer_text: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_adaptive_audit_event_score_view(item: &AdaptiveAuditEventScore) -> AdaptiveAuditEventScoreView {
    AdaptiveAuditEventScoreView {
        code: item.code.clone(),
        name: item.name.clone(),
        source: item.source.clone(),
        checksum: item.checksum.clone(),
        issuer_text: item.issuer.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_audit_event_score_views(items: &[AdaptiveAuditEventScore]) -> Vec<AdaptiveAuditEventScoreView> {
    let mut views: Vec<AdaptiveAuditEventScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_audit_event_score_view(item));
    }
    views
}

pub fn sum_adaptive_audit_event_score_view(view: &AdaptiveAuditEventScoreView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_audit_event_score("code-938", "name-532");
        let view = to_adaptive_audit_event_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_audit_event_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
