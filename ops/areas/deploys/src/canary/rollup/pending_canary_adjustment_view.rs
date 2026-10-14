use super::*;

use acme_kit::text;

/// How screens and exports show a pending canary adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub source: String,
    pub visible: String,
    pub created_at: String,
    pub grace_days: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_pending_canary_adjustment_view(item: &PendingCanaryAdjustment) -> PendingCanaryAdjustmentView {
    PendingCanaryAdjustmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        source: item.source.clone(),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        grace_days: item.grace_days.to_string(),
        tenant_id: item.tenant_id.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_canary_adjustment_views(items: &[PendingCanaryAdjustment]) -> Vec<PendingCanaryAdjustmentView> {
    let mut views: Vec<PendingCanaryAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_canary_adjustment_view(item));
    }
    views
}

pub fn sum_pending_canary_adjustment_view(view: &PendingCanaryAdjustmentView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_canary_adjustment("code-649", "title-865");
        let view = to_pending_canary_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_canary_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
