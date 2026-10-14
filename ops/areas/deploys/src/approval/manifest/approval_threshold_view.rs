use super::*;

use acme_kit::text;

/// How screens and exports show a approval threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub window_days: String,
    pub host: String,
    pub sequence: String,
    pub fingerprint: String,
    pub tenant_id: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_approval_threshold_view(item: &ApprovalThreshold) -> ApprovalThresholdView {
    ApprovalThresholdView {
        code: item.code.clone(),
        label: item.label.clone(),
        external_ref_text: item.external_ref.clone(),
        window_days: item.window_days.to_string(),
        host: item.host.clone(),
        sequence: item.sequence.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        tenant_id: item.tenant_id.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_threshold_views(items: &[ApprovalThreshold]) -> Vec<ApprovalThresholdView> {
    let mut views: Vec<ApprovalThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_threshold_view(item));
    }
    views
}

pub fn sum_approval_threshold_view(view: &ApprovalThresholdView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_threshold("code-719", "label-102");
        let view = to_approval_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
