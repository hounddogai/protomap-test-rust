use super::*;

use acme_kit::text;

/// How screens and exports show a global audit event rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupView {
    pub code: String,
    pub label_text: String,
    pub margin: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_global_audit_event_rollup_view(item: &GlobalAuditEventRollup) -> GlobalAuditEventRollupView {
    GlobalAuditEventRollupView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        scope: text::truncate(&item.scope, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_audit_event_rollup_views(items: &[GlobalAuditEventRollup]) -> Vec<GlobalAuditEventRollupView> {
    let mut views: Vec<GlobalAuditEventRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_audit_event_rollup_view(item));
    }
    views
}

pub fn sum_global_audit_event_rollup_view(view: &GlobalAuditEventRollupView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_audit_event_rollup("code-513", "label-193");
        let view = to_global_audit_event_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_audit_event_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
