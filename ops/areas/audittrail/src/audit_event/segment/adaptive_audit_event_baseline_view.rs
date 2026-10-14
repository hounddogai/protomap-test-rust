use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive audit event baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventBaselineView {
    pub code_text: String,
    pub label_text: String,
    pub country_text: String,
    pub reviewer: String,
    pub maximum: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_adaptive_audit_event_baseline_view(item: &AdaptiveAuditEventBaseline) -> AdaptiveAuditEventBaselineView {
    AdaptiveAuditEventBaselineView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        country_text: item.country.clone(),
        reviewer: item.reviewer.clone(),
        maximum: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_audit_event_baseline_views(
    items: &[AdaptiveAuditEventBaseline],
) -> Vec<AdaptiveAuditEventBaselineView> {
    let mut views: Vec<AdaptiveAuditEventBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_audit_event_baseline_view(item));
    }
    views
}

pub fn sum_adaptive_audit_event_baseline_view(view: &AdaptiveAuditEventBaselineView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_maximum + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_audit_event_baseline("code-631", "label-860");
        let view = to_adaptive_audit_event_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_audit_event_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
