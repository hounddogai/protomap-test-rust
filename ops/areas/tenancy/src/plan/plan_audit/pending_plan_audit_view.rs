use super::*;

use acme_kit::text;

/// How screens and exports show a pending plan audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditView {
    pub code: String,
    pub title_text: String,
    pub floor_value: String,
    pub scope: String,
    pub tenant_id: String,
    pub active: String,
    pub channel_text: String,
    pub external_ref_text: String,
    pub rate: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_pending_plan_audit_view(item: &PendingPlanAudit) -> PendingPlanAuditView {
    PendingPlanAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        scope: text::truncate(&item.scope, 120),
        tenant_id: item.tenant_id.clone(),
        active: format!("{}", item.active),
        channel_text: item.channel.clone(),
        external_ref_text: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_plan_audit_views(items: &[PendingPlanAudit]) -> Vec<PendingPlanAuditView> {
    let mut views: Vec<PendingPlanAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_plan_audit_view(item));
    }
    views
}

pub fn sum_pending_plan_audit_view(view: &PendingPlanAuditView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_floor_value + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_plan_audit("code-403", "title-46");
        let view = to_pending_plan_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_plan_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
