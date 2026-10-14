use super::*;

use acme_kit::text;

/// How screens and exports show a strict approval threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalThresholdView {
    pub code_text: String,
    pub label_text: String,
    pub category: String,
    pub issuer: String,
    pub port_text: String,
    pub title: String,
    pub tenant_id_text: String,
    pub principal: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_strict_approval_threshold_view(item: &StrictApprovalThreshold) -> StrictApprovalThresholdView {
    StrictApprovalThresholdView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        category: item.category.clone(),
        issuer: text::truncate(&item.issuer, 64),
        port_text: item.port.to_string(),
        title: item.title.clone(),
        tenant_id_text: item.tenant_id.clone(),
        principal: item.principal.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_approval_threshold_views(items: &[StrictApprovalThreshold]) -> Vec<StrictApprovalThresholdView> {
    let mut views: Vec<StrictApprovalThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_approval_threshold_view(item));
    }
    views
}

pub fn sum_strict_approval_threshold_view(view: &StrictApprovalThresholdView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_approval_threshold("code-997", "label-828");
        let view = to_strict_approval_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_approval_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
